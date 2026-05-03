//! Canvas editor operations — layout tree mutations for the lowcode platform.
//!
//! Provides `LayoutEditor` with methods to place, update, delete, move, and
//! reparent component nodes within a `LayoutSchema`. Includes nested path
//! resolution, grid collision detection, and comprehensive error reporting.

use crate::schema::{ComponentNode, GridArea, LayoutSchema};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned by canvas editor operations.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum EditorError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("grid conflict: {0}")]
    GridConflict(String),
    #[error("invalid props: {0}")]
    InvalidProps(String),
}

// ---------------------------------------------------------------------------
// Path parsing
// ---------------------------------------------------------------------------

/// Parse a slash-separated path string (e.g. `"0/2/1"`) into child indices.
///
/// An empty string or `"root"` returns `Ok(vec![])` (meaning the top-level
/// children list). Non-numeric segments cause `Err(InvalidPath)`.
pub fn parse_path(path: &str) -> Result<Vec<usize>, EditorError> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "root" {
        return Ok(vec![]);
    }
    trimmed
        .split('/')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| EditorError::InvalidPath(format!("non-numeric segment: {s:?}")))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Grid collision detection
// ---------------------------------------------------------------------------

/// Returns `true` if two grid areas overlap (both axes must intersect).
pub fn has_overlap(a: &GridArea, b: &GridArea) -> bool {
    // Grid areas are inclusive on all boundaries (1-indexed).
    let col_overlap = a.col_start < b.col_end && b.col_start < a.col_end;
    let row_overlap = a.row_start < b.row_end && b.row_start < a.row_end;
    col_overlap && row_overlap
}

/// Check a new grid area against all siblings of `parent`.
///
/// `exclude_id` skips the node being moved (so it doesn't conflict with
/// itself). Returns `Ok(())` if no overlap, `Err(GridConflict)` otherwise.
pub fn check_grid_conflict(
    parent: &ComponentNode,
    new_area: &GridArea,
    exclude_id: Option<&str>,
) -> Result<(), EditorError> {
    for child in &parent.children {
        if let Some(eid) = exclude_id {
            if child.id == eid {
                continue;
            }
        }
        if has_overlap(new_area, &child.grid_area) {
            return Err(EditorError::GridConflict(format!(
                "new area ({},{})→({},{}) overlaps with node {} at ({},{})→({},{})",
                new_area.col_start,
                new_area.row_start,
                new_area.col_end,
                new_area.row_end,
                child.id,
                child.grid_area.col_start,
                child.grid_area.row_start,
                child.grid_area.col_end,
                child.grid_area.row_end,
            )));
        }
    }
    Ok(())
}

/// Variant for checking against the top-level children list.
pub fn check_grid_conflict_root(
    children: &[ComponentNode],
    new_area: &GridArea,
    exclude_id: Option<&str>,
) -> Result<(), EditorError> {
    for child in children {
        if let Some(eid) = exclude_id {
            if child.id == eid {
                continue;
            }
        }
        if has_overlap(new_area, &child.grid_area) {
            return Err(EditorError::GridConflict(format!(
                "new area ({},{})→({},{}) overlaps with node {} at ({},{})→({},{})",
                new_area.col_start,
                new_area.row_start,
                new_area.col_end,
                new_area.row_end,
                child.id,
                child.grid_area.col_start,
                child.grid_area.row_start,
                child.grid_area.col_end,
                child.grid_area.row_end,
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// LayoutEditor — stateless canvas mutation API
// ---------------------------------------------------------------------------

/// Stateless editor that mutates a `LayoutSchema` tree.
pub struct LayoutEditor;

impl LayoutEditor {
    /// Place a new component node under the parent identified by `parent_path`.
    ///
    /// - `parent_path`: `"root"` or `""` for top-level; `"0/2"` for nested.
    /// - Returns the newly generated node id.
    pub fn place_component(
        layout: &mut LayoutSchema,
        parent_path: &str,
        component_type: &str,
        grid_area: GridArea,
        props: serde_json::Value,
    ) -> Result<String, EditorError> {
        let indices = parse_path(parent_path)?;
        let parent_children = resolve_children_mut(layout, &indices)?;
        check_grid_conflict_root(parent_children, &grid_area, None)?;

        let id = uuid::Uuid::new_v4().to_string();
        let node = ComponentNode {
            id: id.clone(),
            type_key: component_type.to_string(),
            props,
            grid_area,
            children: vec![],
        };
        parent_children.push(node);
        Ok(id)
    }

    /// Merge `props_patch` into the existing props of the node at `path`.
    pub fn update_props(
        layout: &mut LayoutSchema,
        path: &str,
        props_patch: serde_json::Value,
    ) -> Result<(), EditorError> {
        let indices = parse_path(path)?;
        let node = resolve_node_mut(layout, &indices)?;
        merge_json(&mut node.props, props_patch);
        Ok(())
    }

    /// Remove the node at `path` from its parent and return it.
    pub fn delete_component(
        layout: &mut LayoutSchema,
        path: &str,
    ) -> Result<ComponentNode, EditorError> {
        let indices = parse_path(path)?;
        if indices.is_empty() {
            return Err(EditorError::InvalidPath(
                "cannot delete root — provide a child path".into(),
            ));
        }
        let (parent_children, idx) = resolve_parent_and_index(layout, &indices)?;
        if idx >= parent_children.len() {
            return Err(EditorError::NotFound(format!(
                "index {idx} out of bounds (len {})",
                parent_children.len()
            )));
        }
        Ok(parent_children.remove(idx))
    }

    /// Change the grid area of the node at `path`, checking for conflicts.
    pub fn move_component(
        layout: &mut LayoutSchema,
        path: &str,
        new_grid_area: GridArea,
    ) -> Result<(), EditorError> {
        let indices = parse_path(path)?;
        if indices.is_empty() {
            return Err(EditorError::InvalidPath(
                "cannot move root — provide a child path".into(),
            ));
        }
        let node_id = {
            let node = resolve_node_mut(layout, &indices)?;
            node.id.clone()
        };

        // Check grid conflict at the parent level, excluding self.
        let parent_indices = &indices[..indices.len() - 1];
        let parent_children = resolve_children_mut(layout, parent_indices)?;
        check_grid_conflict_root(parent_children, &new_grid_area, Some(&node_id))?;

        let node = parent_children
            .get_mut(*indices.last().unwrap())
            .ok_or_else(|| EditorError::NotFound(format!("index {} out of bounds", indices.last().unwrap())))?;
        node.grid_area = new_grid_area;
        Ok(())
    }

    /// Remove node at `path` and re-insert under the parent at `new_parent_path`.
    pub fn reparent_component(
        layout: &mut LayoutSchema,
        path: &str,
        new_parent_path: &str,
        new_grid_area: GridArea,
    ) -> Result<(), EditorError> {
        let indices = parse_path(path)?;
        if indices.is_empty() {
            return Err(EditorError::InvalidPath(
                "cannot reparent root — provide a child path".into(),
            ));
        }

        // Remove node from old location.
        let removed = Self::delete_component(layout, path)?;

        // Insert at new parent.
        let new_indices = parse_path(new_parent_path)?;
        let target_children = match resolve_children_mut(layout, &new_indices) {
            Ok(c) => c,
            Err(e) => {
                // Put the node back on failure.
                let old_indices = parse_path(path).unwrap_or_default();
                if let Ok(old_children) = resolve_children_mut(layout, &old_indices) {
                    old_children.push(removed);
                }
                return Err(e);
            }
        };

        if let Err(e) = check_grid_conflict_root(target_children, &new_grid_area, None) {
            // Put the node back on failure.
            let old_indices = parse_path(path).unwrap_or_default();
            if let Ok(old_children) = resolve_children_mut(layout, &old_indices) {
                old_children.push(removed);
            }
            return Err(e);
        }

        let mut node = removed;
        node.grid_area = new_grid_area;
        target_children.push(node);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Resolve the `&mut Vec<ComponentNode>` at the given index path.
///
/// An empty `indices` slice returns `layout.children`.
fn resolve_children_mut<'a>(
    layout: &'a mut LayoutSchema,
    indices: &[usize],
) -> Result<&'a mut Vec<ComponentNode>, EditorError> {
    let mut children = &mut layout.children;
    for (depth, &idx) in indices.iter().enumerate() {
        if idx >= children.len() {
            return Err(EditorError::NotFound(format!(
                "index {idx} out of bounds at depth {depth} (len {})",
                children.len()
            )));
        }
        children = &mut children[idx].children;
    }
    Ok(children)
}

/// Resolve a `&mut ComponentNode` at the given index path.
fn resolve_node_mut<'a>(
    layout: &'a mut LayoutSchema,
    indices: &[usize],
) -> Result<&'a mut ComponentNode, EditorError> {
    if indices.is_empty() {
        return Err(EditorError::InvalidPath(
            "path resolves to root (no node at empty path)".into(),
        ));
    }
    let (parent_indices, last_idx) = indices.split_at(indices.len() - 1);
    let last = last_idx[0];
    let children = resolve_children_mut(layout, parent_indices)?;
    if last >= children.len() {
        return Err(EditorError::NotFound(format!(
            "index {last} out of bounds (len {})",
            children.len()
        )));
    }
    Ok(&mut children[last])
}

/// Resolve parent `Vec<ComponentNode>` and child index from a non-empty path.
fn resolve_parent_and_index<'a>(
    layout: &'a mut LayoutSchema,
    indices: &[usize],
) -> Result<(&'a mut Vec<ComponentNode>, usize), EditorError> {
    if indices.is_empty() {
        return Err(EditorError::InvalidPath(
            "path resolves to root — no parent".into(),
        ));
    }
    let (parent_indices, last_idx) = indices.split_at(indices.len() - 1);
    let last = last_idx[0];
    let children = resolve_children_mut(layout, parent_indices)?;
    Ok((children, last))
}

/// Shallow-merge `patch` into `target` (only top-level keys for JSON objects).
fn merge_json(target: &mut serde_json::Value, patch: serde_json::Value) {
    if let (serde_json::Value::Object(target_map), serde_json::Value::Object(patch_map)) =
        (target, patch)
    {
        for (k, v) in patch_map {
            target_map.insert(k, v);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{GridArea, GridDefinition, LayoutSchema};

    fn empty_layout() -> LayoutSchema {
        LayoutSchema {
            grid: GridDefinition {
                columns: 12,
                rows: 8,
                gap: None,
            },
            children: vec![],
        }
    }

    fn area(cs: u32, rs: u32, ce: u32, re: u32) -> GridArea {
        GridArea {
            col_start: cs,
            col_end: ce,
            row_start: rs,
            row_end: re,
        }
    }

    // ---- parse_path tests ----

    #[test]
    fn test_parse_path_valid() {
        assert_eq!(parse_path("").unwrap(), Vec::<usize>::new());
        assert_eq!(parse_path("root").unwrap(), Vec::<usize>::new());
        assert_eq!(parse_path("0").unwrap(), vec![0]);
        assert_eq!(parse_path("0/2/1").unwrap(), vec![0, 2, 1]);
        assert_eq!(parse_path(" 3 ").unwrap(), vec![3]);
    }

    #[test]
    fn test_parse_path_invalid() {
        assert!(parse_path("0/abc/1").is_err());
        assert!(parse_path("foo").is_err());
    }

    // ---- place_component tests ----

    #[test]
    fn test_place_component_at_root() {
        let mut layout = empty_layout();
        let id = LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 2),
            serde_json::json!({ "label": "OK" }),
        )
        .unwrap();

        assert_eq!(layout.children.len(), 1);
        let node = &layout.children[0];
        assert_eq!(node.id, id);
        assert_eq!(node.type_key, "button");
        assert_eq!(node.props["label"], "OK");
        assert_eq!(node.grid_area, area(1, 1, 3, 2));
    }

    #[test]
    fn test_place_component_nested() {
        let mut layout = empty_layout();

        // Place a container at root.
        let container_id = LayoutEditor::place_component(
            &mut layout,
            "root",
            "container",
            area(1, 1, 13, 9),
            serde_json::json!({}),
        )
        .unwrap();
        assert!(!container_id.is_empty());

        // Place a button inside the container (path "0").
        let btn_id = LayoutEditor::place_component(
            &mut layout,
            "0",
            "button",
            area(1, 1, 3, 2),
            serde_json::json!({ "label": "Click" }),
        )
        .unwrap();

        assert_eq!(layout.children[0].children.len(), 1);
        assert_eq!(layout.children[0].children[0].id, btn_id);
        assert_eq!(layout.children[0].children[0].type_key, "button");
    }

    // ---- update_props tests ----

    #[test]
    fn test_update_props() {
        let mut layout = empty_layout();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 2),
            serde_json::json!({ "label": "Old", "variant": "primary" }),
        )
        .unwrap();

        LayoutEditor::update_props(
            &mut layout,
            "0",
            serde_json::json!({ "label": "New" }),
        )
        .unwrap();

        let node = &layout.children[0];
        assert_eq!(node.props["label"], "New");
        assert_eq!(node.props["variant"], "primary"); // unchanged
    }

    // ---- delete_component tests ----

    #[test]
    fn test_delete_component() {
        let mut layout = empty_layout();
        let id = LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 2),
            serde_json::json!({}),
        )
        .unwrap();

        let removed = LayoutEditor::delete_component(&mut layout, "0").unwrap();
        assert_eq!(removed.id, id);
        assert_eq!(removed.type_key, "button");
        assert!(layout.children.is_empty());
    }

    #[test]
    fn test_delete_component_nested() {
        let mut layout = empty_layout();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "container",
            area(1, 1, 13, 9),
            serde_json::json!({}),
        )
        .unwrap();
        LayoutEditor::place_component(
            &mut layout,
            "0",
            "text",
            area(1, 1, 2, 2),
            serde_json::json!({ "content": "hi" }),
        )
        .unwrap();
        LayoutEditor::place_component(
            &mut layout,
            "0",
            "button",
            area(2, 1, 3, 2),
            serde_json::json!({ "label": "Btn" }),
        )
        .unwrap();

        assert_eq!(layout.children[0].children.len(), 2);
        let removed = LayoutEditor::delete_component(&mut layout, "0/0").unwrap();
        assert_eq!(removed.type_key, "text");
        assert_eq!(layout.children[0].children.len(), 1);
        assert_eq!(layout.children[0].children[0].type_key, "button");
    }

    // ---- move_component tests ----

    #[test]
    fn test_move_component() {
        let mut layout = empty_layout();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 2),
            serde_json::json!({}),
        )
        .unwrap();

        LayoutEditor::move_component(&mut layout, "0", area(5, 5, 7, 6)).unwrap();

        assert_eq!(layout.children[0].grid_area, area(5, 5, 7, 6));
    }

    // ---- reparent_component tests ----

    #[test]
    fn test_reparent_component() {
        let mut layout = empty_layout();

        // Two root containers.
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "container",
            area(1, 1, 7, 9),
            serde_json::json!({}),
        )
        .unwrap();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "container",
            area(7, 1, 13, 9),
            serde_json::json!({}),
        )
        .unwrap();

        // A button inside container 0.
        LayoutEditor::place_component(
            &mut layout,
            "0",
            "button",
            area(1, 1, 2, 2),
            serde_json::json!({ "label": "Move me" }),
        )
        .unwrap();

        assert_eq!(layout.children[0].children.len(), 1);
        assert_eq!(layout.children[1].children.len(), 0);

        // Move button from container 0 → container 1.
        LayoutEditor::reparent_component(
            &mut layout,
            "0/0",
            "1",
            area(1, 1, 2, 2),
        )
        .unwrap();

        assert_eq!(layout.children[0].children.len(), 0);
        assert_eq!(layout.children[1].children.len(), 1);
        assert_eq!(layout.children[1].children[0].type_key, "button");
    }

    // ---- grid conflict tests ----

    #[test]
    fn test_grid_conflict_detection() {
        let mut layout = empty_layout();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 3),
            serde_json::json!({}),
        )
        .unwrap();

        // Overlapping placement should fail.
        let result = LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(2, 2, 4, 4),
            serde_json::json!({}),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::GridConflict(msg) => assert!(msg.contains("overlaps")),
            other => panic!("expected GridConflict, got {other:?}"),
        }
    }

    #[test]
    fn test_grid_no_conflict_non_overlapping() {
        let mut layout = empty_layout();
        LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(1, 1, 3, 3),
            serde_json::json!({}),
        )
        .unwrap();

        // Non-overlapping should succeed.
        let id = LayoutEditor::place_component(
            &mut layout,
            "root",
            "button",
            area(3, 1, 5, 3),
            serde_json::json!({}),
        )
        .unwrap();
        assert!(!id.is_empty());
        assert_eq!(layout.children.len(), 2);
    }

    // ---- error / not-found tests ----

    #[test]
    fn test_invalid_path() {
        let mut layout = empty_layout();
        let result = LayoutEditor::update_props(
            &mut layout,
            "0/abc",
            serde_json::json!({}),
        );
        assert!(matches!(result, Err(EditorError::InvalidPath(_))));
    }

    #[test]
    fn test_not_found() {
        let mut layout = empty_layout();
        let result = LayoutEditor::delete_component(&mut layout, "0");
        assert!(matches!(result, Err(EditorError::NotFound(_))));
    }

    #[test]
    fn test_update_props_not_found() {
        let mut layout = empty_layout();
        let result = LayoutEditor::update_props(
            &mut layout,
            "0",
            serde_json::json!({ "label": "X" }),
        );
        assert!(matches!(result, Err(EditorError::NotFound(_))));
    }

    #[test]
    fn test_move_not_found() {
        let mut layout = empty_layout();
        let result = LayoutEditor::move_component(&mut layout, "0", area(1, 1, 2, 2));
        assert!(matches!(result, Err(EditorError::NotFound(_))));
    }

    #[test]
    fn test_reparent_not_found_source() {
        let mut layout = empty_layout();
        let result = LayoutEditor::reparent_component(
            &mut layout,
            "0",
            "root",
            area(1, 1, 2, 2),
        );
        assert!(matches!(result, Err(EditorError::NotFound(_))));
    }

    #[test]
    fn test_delete_root_path() {
        let mut layout = empty_layout();
        let result = LayoutEditor::delete_component(&mut layout, "root");
        assert!(matches!(result, Err(EditorError::InvalidPath(_))));
    }

    #[test]
    fn test_move_root_path() {
        let mut layout = empty_layout();
        let result = LayoutEditor::move_component(&mut layout, "root", area(1, 1, 2, 2));
        assert!(matches!(result, Err(EditorError::InvalidPath(_))));
    }

    // ---- has_overlap edge cases ----

    #[test]
    fn test_has_overlap_adjacent_no_conflict() {
        // Adjacent (touching boundary) should NOT overlap.
        let a = area(1, 1, 3, 3);
        let b = area(3, 1, 5, 3);
        assert!(!has_overlap(&a, &b));
    }

    #[test]
    fn test_has_overlap_identical() {
        let a = area(1, 1, 3, 3);
        assert!(has_overlap(&a, &a));
    }

    #[test]
    fn test_has_overlap_contained() {
        let outer = area(1, 1, 5, 5);
        let inner = area(2, 2, 4, 4);
        assert!(has_overlap(&outer, &inner));
    }
}
