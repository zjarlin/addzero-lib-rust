use crate::schema::{Breakpoint, ComponentNode, GridArea, GridDefinition, LayoutSchema};

/// Default grid column count used when a schema provides an invalid zero value.
pub const DEFAULT_COLUMNS: u16 = 12;

/// Grid-based layout engine that maps layout schema nodes onto CSS Grid rules.
#[derive(Debug, Clone, Copy)]
pub struct GridEngine;

impl GridEngine {
    pub const fn new() -> Self {
        Self
    }

    pub fn compile_css(layout: &LayoutSchema) -> String {
        Self::new().render_css(layout)
    }

    pub fn render_css(&self, layout: &LayoutSchema) -> String {
        let mut blocks = vec![render_canvas_rule(&layout.grid)];
        let mut nested_container_selectors = Vec::new();

        collect_node_blocks(
            &layout.children,
            &layout.grid,
            &mut blocks,
            &mut nested_container_selectors,
        );

        for breakpoint in &layout.grid.breakpoints {
            blocks.push(render_breakpoint_block(
                breakpoint,
                &layout.grid,
                &nested_container_selectors,
            ));
        }

        blocks.join("\n\n")
    }
}

impl Default for GridEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compile_css(layout: &LayoutSchema) -> String {
    GridEngine::compile_css(layout)
}

impl GridArea {
    pub fn to_css(&self) -> String {
        format!(
            "grid-column: {} / {}; grid-row: {} / {};",
            self.col_start, self.col_end, self.row_start, self.row_end
        )
    }
}

fn collect_node_blocks(
    nodes: &[ComponentNode],
    grid: &GridDefinition,
    blocks: &mut Vec<String>,
    nested_container_selectors: &mut Vec<String>,
) {
    for node in nodes {
        if !node.children.is_empty() {
            nested_container_selectors.push(node_selector(&node.id));
        }

        blocks.push(render_node_rule(node, grid));
        collect_node_blocks(&node.children, grid, blocks, nested_container_selectors);
    }
}

fn render_canvas_rule(grid: &GridDefinition) -> String {
    render_rule(".lc-canvas", grid_properties(grid, true))
}

fn render_node_rule(node: &ComponentNode, grid: &GridDefinition) -> String {
    let mut declarations = vec![node.grid_area.to_css()];

    if !node.children.is_empty() {
        declarations.extend(grid_properties(grid, true));
    }

    render_rule(&node_selector(&node.id), declarations)
}

fn render_breakpoint_block(
    breakpoint: &Breakpoint,
    base_grid: &GridDefinition,
    nested_container_selectors: &[String],
) -> String {
    let mut rules = Vec::with_capacity(nested_container_selectors.len() + 1);
    let declarations = breakpoint_properties(breakpoint, base_grid);

    rules.push(indent_block(&render_rule(
        ".lc-canvas",
        declarations.clone(),
    )));

    for selector in nested_container_selectors {
        rules.push(indent_block(&render_rule(selector, declarations.clone())));
    }

    format!(
        "@media (max-width: {}) {{\n{}\n}}",
        breakpoint.max_width,
        rules.join("\n\n")
    )
}

fn grid_properties(grid: &GridDefinition, include_display: bool) -> Vec<String> {
    let mut declarations = Vec::with_capacity(4);

    if include_display {
        declarations.push("display: grid;".to_string());
    }

    declarations.push(format!(
        "grid-template-columns: repeat({}, minmax(0, 1fr));",
        resolved_columns(grid.columns)
    ));
    declarations.push(format!(
        "grid-template-rows: repeat({}, {});",
        resolved_rows(grid.rows),
        row_track_size(grid.row_height.as_deref())
    ));

    if let Some(gap) = grid.gap.as_deref() {
        declarations.push(format!("gap: {gap};"));
    }

    declarations
}

fn breakpoint_properties(breakpoint: &Breakpoint, base_grid: &GridDefinition) -> Vec<String> {
    let mut declarations = Vec::with_capacity(3);

    declarations.push(format!(
        "grid-template-columns: repeat({}, minmax(0, 1fr));",
        resolved_breakpoint_columns(breakpoint.columns)
    ));

    if let Some(row_height) = breakpoint.row_height.as_deref() {
        declarations.push(format!(
            "grid-template-rows: repeat({}, {});",
            resolved_rows(base_grid.rows),
            row_height
        ));
    }

    if let Some(gap) = breakpoint.gap.as_deref() {
        declarations.push(format!("gap: {gap};"));
    }

    declarations
}

fn render_rule(selector: &str, declarations: Vec<String>) -> String {
    let mut css = String::new();
    css.push_str(selector);
    css.push_str(" {\n");

    for declaration in declarations {
        css.push_str("  ");
        css.push_str(&declaration);
        if !declaration.ends_with(';') {
            css.push(';');
        }
        css.push('\n');
    }

    css.push('}');
    css
}

fn indent_block(block: &str) -> String {
    let mut indented = String::new();

    for line in block.lines() {
        indented.push_str("  ");
        indented.push_str(line);
        indented.push('\n');
    }

    indented.pop();
    indented
}

fn node_selector(node_id: &str) -> String {
    format!(".lc-node-{node_id}")
}

fn row_track_size(row_height: Option<&str>) -> &str {
    row_height.unwrap_or("minmax(0, auto)")
}

fn resolved_columns(columns: u32) -> u32 {
    if columns == 0 {
        u32::from(DEFAULT_COLUMNS)
    } else {
        columns
    }
}

fn resolved_breakpoint_columns(columns: u16) -> u16 {
    if columns == 0 {
        DEFAULT_COLUMNS
    } else {
        columns
    }
}

fn resolved_rows(rows: u32) -> u32 {
    rows.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Breakpoint, ComponentNode};

    #[test]
    fn grid_area_to_css_should_render_basic_range() {
        let area = GridArea {
            col_start: 1,
            col_end: 3,
            row_start: 2,
            row_end: 4,
        };

        assert_eq!(area.to_css(), "grid-column: 1 / 3; grid-row: 2 / 4;");
    }

    #[test]
    fn grid_area_to_css_should_render_various_ranges() {
        let area = GridArea {
            col_start: 3,
            col_end: 7,
            row_start: 1,
            row_end: 9,
        };

        assert_eq!(area.to_css(), "grid-column: 3 / 7; grid-row: 1 / 9;");
    }

    #[test]
    fn compile_css_should_render_simple_two_node_layout() {
        let layout = layout_with_children(
            base_grid_definition(),
            vec![
                node("header", 1, 13, 1, 2, vec![]),
                node("content", 1, 13, 2, 7, vec![]),
            ],
        );

        let css = compile_css(&layout);

        assert!(css.contains(".lc-canvas {"));
        assert!(css.contains("display: grid;"));
        assert!(css.contains("grid-template-columns: repeat(12, minmax(0, 1fr));"));
        assert!(css.contains("grid-template-rows: repeat(6, 80px);"));
        assert!(css.contains("gap: 16px;"));
        assert!(css.contains(".lc-node-header {"));
        assert!(css.contains("grid-column: 1 / 13; grid-row: 1 / 2;"));
        assert!(css.contains(".lc-node-content {"));
        assert!(css.contains("grid-column: 1 / 13; grid-row: 2 / 7;"));
    }

    #[test]
    fn compile_css_should_render_breakpoint_media_queries() {
        let mut grid = base_grid_definition();
        grid.breakpoints = vec![
            Breakpoint {
                name: "tablet".into(),
                max_width: "1024px".into(),
                columns: 8,
                row_height: Some("64px".into()),
                gap: Some("12px".into()),
            },
            Breakpoint {
                name: "mobile".into(),
                max_width: "640px".into(),
                columns: 4,
                row_height: None,
                gap: None,
            },
        ];

        let layout = layout_with_children(grid, vec![node("hero", 1, 13, 1, 4, vec![])]);
        let css = GridEngine::compile_css(&layout);

        assert!(css.contains("@media (max-width: 1024px) {"));
        assert!(css.contains("grid-template-columns: repeat(8, minmax(0, 1fr));"));
        assert!(css.contains("grid-template-rows: repeat(6, 64px);"));
        assert!(css.contains("gap: 12px;"));
        assert!(css.contains("@media (max-width: 640px) {"));
        assert!(css.contains("grid-template-columns: repeat(4, minmax(0, 1fr));"));
    }

    #[test]
    fn compile_css_should_render_nested_container_grids() {
        let layout = layout_with_children(
            base_grid_definition(),
            vec![node(
                "container",
                1,
                13,
                1,
                7,
                vec![
                    node("child-a", 1, 7, 1, 3, vec![]),
                    node("child-b", 7, 13, 1, 3, vec![]),
                ],
            )],
        );

        let css = compile_css(&layout);

        assert!(css.contains(".lc-node-container {"));
        assert!(css.contains(".lc-node-child-a {"));
        assert!(css.contains(".lc-node-child-b {"));
        assert!(css.contains(
            ".lc-node-container {\n  grid-column: 1 / 13; grid-row: 1 / 7;\n  display: grid;"
        ));
        assert!(css.contains(".lc-node-child-a {\n  grid-column: 1 / 7; grid-row: 1 / 3;"));
    }

    #[test]
    fn compile_css_should_handle_empty_children() {
        let layout = layout_with_children(base_grid_definition(), vec![]);

        let css = compile_css(&layout);

        assert!(css.contains(".lc-canvas {"));
        assert!(!css.contains(".lc-node-"));
    }

    #[test]
    fn default_columns_should_match_expected_grid_width() {
        assert_eq!(DEFAULT_COLUMNS, 12);
    }

    fn base_grid_definition() -> GridDefinition {
        GridDefinition {
            columns: 12,
            rows: 6,
            gap: Some("16px".into()),
            row_height: Some("80px".into()),
            breakpoints: vec![],
        }
    }

    fn layout_with_children(grid: GridDefinition, children: Vec<ComponentNode>) -> LayoutSchema {
        LayoutSchema { grid, children }
    }

    fn node(
        id: &str,
        col_start: u32,
        col_end: u32,
        row_start: u32,
        row_end: u32,
        children: Vec<ComponentNode>,
    ) -> ComponentNode {
        ComponentNode {
            id: id.into(),
            type_key: "container".into(),
            props: serde_json::json!({}),
            grid_area: GridArea {
                col_start,
                col_end,
                row_start,
                row_end,
            },
            children,
        }
    }
}
