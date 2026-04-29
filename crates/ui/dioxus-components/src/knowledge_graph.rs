use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalized(self) -> Self {
        let len = self.length();
        if len <= f32::EPSILON {
            Self::ZERO
        } else {
            Self::new(self.x / len, self.y / len)
        }
    }

    fn clamp(self, width: f32, height: f32, padding: f32) -> Self {
        Self::new(
            self.x.clamp(padding, width - padding),
            self.y.clamp(padding, height - padding),
        )
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GraphNodeCategory {
    Service,
    Database,
    Knowledge,
    Workflow,
    File,
    Person,
}

impl GraphNodeCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Service => "服务",
            Self::Database => "数据",
            Self::Knowledge => "知识",
            Self::Workflow => "流程",
            Self::File => "文件",
            Self::Person => "角色",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Self::Service => "#5bd1ff",
            Self::Database => "#77e0a0",
            Self::Knowledge => "#ffd36a",
            Self::Workflow => "#ff8a78",
            Self::File => "#b49bff",
            Self::Person => "#ffb86b",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEdgeKind {
    DependsOn,
    Emits,
    Reads,
    Owns,
    RelatesTo,
}

impl GraphEdgeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::DependsOn => "依赖",
            Self::Emits => "产出",
            Self::Reads => "读取",
            Self::Owns => "负责",
            Self::RelatesTo => "关联",
        }
    }

    fn stroke(self) -> &'static str {
        match self {
            Self::DependsOn => "#5bd1ff",
            Self::Emits => "#77e0a0",
            Self::Reads => "#ffd36a",
            Self::Owns => "#ff8a78",
            Self::RelatesTo => "#8b99ab",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub category: GraphNodeCategory,
    pub description: String,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub kind: GraphEdgeKind,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilteredGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[component]
pub fn KnowledgeGraph(
    graph: GraphData,
    #[props(optional)] title: Option<String>,
    #[props(optional)] subtitle: Option<String>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut selected_categories = use_signal(BTreeSet::<GraphNodeCategory>::new);
    let mut focused_node_id = use_signal(|| graph.nodes.first().map(|node| node.id.clone()));

    let filtered = filter_graph(&graph, &selected_categories.read(), &search.read(), 1);

    let positions = force_layout(&filtered, 960.0, 560.0, 160);
    let focused = focused_node_id
        .read()
        .as_ref()
        .and_then(|id| filtered.nodes.iter().find(|node| &node.id == id))
        .cloned()
        .or_else(|| filtered.nodes.first().cloned());

    let summary = format!(
        "{} 节点 / {} 连线",
        filtered.nodes.len(),
        filtered.edges.len()
    );
    let title = title.unwrap_or_else(|| "知识图谱".to_string());
    let subtitle =
        subtitle.unwrap_or_else(|| "支持关键词搜索、类型过滤和节点详情联动。".to_string());
    let categories = categories_in(&graph);
    let edge_items: Vec<_> = filtered
        .edges
        .iter()
        .filter_map(|edge| {
            let source = positions.get(&edge.source)?;
            let target = positions.get(&edge.target)?;
            Some((
                format!("{}-{}", edge.source, edge.target),
                *source,
                *target,
                edge.kind.stroke(),
                edge.label
                    .clone()
                    .unwrap_or_else(|| edge.kind.label().to_string()),
            ))
        })
        .collect();
    let node_items: Vec<_> = filtered
        .nodes
        .iter()
        .filter_map(|node| {
            let position = positions.get(&node.id)?;
            let is_focused = focused
                .as_ref()
                .is_some_and(|focused| focused.id == node.id);
            Some((
                node.id.clone(),
                node.label.clone(),
                node.category,
                *position,
                is_focused,
            ))
        })
        .collect();

    rsx! {
        section { class: "kg",
            div { class: "kg__header",
                div {
                    h3 { class: "kg__title", "{title}" }
                    p { class: "kg__subtitle", "{subtitle}" }
                }
                div { class: "kg__summary", "{summary}" }
            }
            div { class: "kg__toolbar",
                input {
                    class: "kg__search",
                    r#type: "search",
                    value: "{search}",
                    placeholder: "搜索节点、描述或详情",
                    oninput: move |evt| search.set(evt.value())
                }
                button {
                    class: "kg__clear",
                    r#type: "button",
                    onclick: move |_| {
                        search.set(String::new());
                        selected_categories.set(BTreeSet::new());
                    },
                    "清空"
                }
            }
            div { class: "kg__chips",
                for category in categories {
                    {
                        let checked = selected_categories.read().contains(&category);
                        let color = category.color();
                        rsx! {
                            button {
                                key: "{category.label()}",
                                class: if checked { "kg__chip kg__chip--active" } else { "kg__chip" },
                                style: format!("--kg-chip-color: {color};"),
                                r#type: "button",
                                onclick: move |_| {
                                    let mut next = selected_categories.read().clone();
                                    if !next.insert(category) {
                                        next.remove(&category);
                                    }
                                    selected_categories.set(next);
                                },
                                span { class: "kg__chip-dot" }
                                "{category.label()}"
                            }
                        }
                    }
                }
            }
            div { class: "kg__layout",
                div { class: "kg__canvas",
                    svg {
                        class: "kg__svg",
                        view_box: "0 0 960 560",
                        preserve_aspect_ratio: "xMidYMid meet",
                        for (key, source, target, stroke, label) in edge_items {
                            g { key: "{key}",
                                line {
                                    class: "kg__edge",
                                    x1: "{source.x}",
                                    y1: "{source.y}",
                                    x2: "{target.x}",
                                    y2: "{target.y}",
                                    stroke: "{stroke}",
                                }
                                text {
                                    class: "kg__edge-label",
                                    x: "{(source.x + target.x) / 2.0}",
                                    y: "{(source.y + target.y) / 2.0}",
                                    text_anchor: "middle",
                                    dominant_baseline: "central",
                                    "{label}"
                                }
                            }
                        }
                        for (node_id, label, category, position, is_focused) in node_items {
                            g {
                                key: "{node_id}",
                                class: if is_focused { "kg__node kg__node--focused" } else { "kg__node" },
                                onclick: move |_| focused_node_id.set(Some(node_id.clone())),
                                circle {
                                    cx: "{position.x}",
                                    cy: "{position.y}",
                                    r: if is_focused { "30" } else { "25" },
                                    fill: "{category.color()}",
                                }
                                text {
                                    class: "kg__node-label",
                                    x: "{position.x}",
                                    y: "{position.y - 1.0}",
                                    text_anchor: "middle",
                                    dominant_baseline: "central",
                                    "{label}"
                                }
                                text {
                                    class: "kg__node-caption",
                                    x: "{position.x}",
                                    y: "{position.y + 44.0}",
                                    text_anchor: "middle",
                                    "{category.label()}"
                                }
                            }
                        }
                    }
                }
                div { class: "kg__detail",
                    if let Some(node) = focused {
                        h4 { class: "kg__detail-title", "{node.label}" }
                        p { class: "kg__detail-category",
                            span {
                                class: "kg__legend-dot",
                                style: format!("background: {};", node.category.color())
                            }
                            "{node.category.label()}"
                        }
                        p { class: "kg__detail-copy", "{node.description}" }
                        p { class: "kg__detail-copy kg__detail-copy--muted", "{node.details}" }
                        div { class: "kg__detail-meta",
                            span { class: "kg__detail-pill", "节点 ID: {node.id}" }
                            span { class: "kg__detail-pill", "相连边: {count_edges(&filtered, &node.id)}" }
                        }
                    } else {
                        h4 { class: "kg__detail-title", "没有匹配结果" }
                        p { class: "kg__detail-copy kg__detail-copy--muted", "调整筛选器或搜索词后，图谱会自动回填。"}
                    }
                }
            }
        }
    }
}

fn categories_in(graph: &GraphData) -> Vec<GraphNodeCategory> {
    let mut categories = BTreeSet::new();
    for node in &graph.nodes {
        categories.insert(node.category);
    }
    categories.into_iter().collect()
}

fn count_edges(graph: &FilteredGraph, node_id: &str) -> usize {
    graph
        .edges
        .iter()
        .filter(|edge| edge.source == node_id || edge.target == node_id)
        .count()
}

fn filter_graph(
    graph: &GraphData,
    selected_categories: &BTreeSet<GraphNodeCategory>,
    search: &str,
    bfs_depth: usize,
) -> FilteredGraph {
    let mut visible_nodes: Vec<GraphNode> = graph
        .nodes
        .iter()
        .filter(|node| {
            selected_categories.is_empty() || selected_categories.contains(&node.category)
        })
        .cloned()
        .collect();

    let search = search.trim().to_lowercase();
    if !search.is_empty() {
        let matched_ids: Vec<String> = visible_nodes
            .iter()
            .filter(|node| matches_search(node, &search))
            .map(|node| node.id.clone())
            .collect();
        let expanded = {
            let visible_ids: HashSet<String> =
                visible_nodes.iter().map(|node| node.id.clone()).collect();
            let adjacency = adjacency_map(&graph.edges, &visible_ids);
            expand_matches(&adjacency, &matched_ids, bfs_depth)
        };
        visible_nodes.retain(|node| expanded.contains(node.id.as_str()));
    }

    let visible_lookup: HashSet<&str> = visible_nodes.iter().map(|node| node.id.as_str()).collect();
    let edges = graph
        .edges
        .iter()
        .filter(|edge| {
            visible_lookup.contains(edge.source.as_str())
                && visible_lookup.contains(edge.target.as_str())
        })
        .cloned()
        .collect();

    FilteredGraph {
        nodes: visible_nodes,
        edges,
    }
}

fn matches_search(node: &GraphNode, search: &str) -> bool {
    node.label.to_lowercase().contains(search)
        || node.description.to_lowercase().contains(search)
        || node.details.to_lowercase().contains(search)
}

fn adjacency_map(
    edges: &[GraphEdge],
    visible_ids: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        if visible_ids.contains(edge.source.as_str()) && visible_ids.contains(edge.target.as_str())
        {
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            adjacency
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }
    }
    adjacency
}

fn expand_matches(
    adjacency: &HashMap<String, Vec<String>>,
    matches: &[String],
    bfs_depth: usize,
) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    for id in matches {
        if visited.insert(id.clone()) {
            queue.push_back((id.clone(), 0usize));
        }
    }

    while let Some((node, depth)) = queue.pop_front() {
        if depth >= bfs_depth {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&node) {
            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
    }

    visited
}

fn force_layout(
    graph: &FilteredGraph,
    width: f32,
    height: f32,
    iterations: usize,
) -> HashMap<String, Vec2> {
    let node_count = graph.nodes.len();
    if node_count == 0 {
        return HashMap::new();
    }

    let padding = 56.0;
    let mut positions: HashMap<String, Vec2> = graph
        .nodes
        .iter()
        .map(|node| {
            let seed_x = seeded_unit(&(node.id.clone() + "-x"));
            let seed_y = seeded_unit(&(node.id.clone() + "-y"));
            let x = padding + seed_x * (width - padding * 2.0);
            let y = padding + seed_y * (height - padding * 2.0);
            (node.id.clone(), Vec2::new(x, y))
        })
        .collect();

    let area = (width - padding * 2.0) * (height - padding * 2.0);
    let ideal_length = (area / node_count as f32).sqrt().clamp(90.0, 180.0);
    let mut temperature = ideal_length * 0.65;
    let center = Vec2::new(width * 0.5, height * 0.5);
    let gravity = 0.018;

    for step in 0..iterations {
        let mut forces: HashMap<String, Vec2> = graph
            .nodes
            .iter()
            .map(|node| (node.id.clone(), Vec2::ZERO))
            .collect();

        for left in 0..graph.nodes.len() {
            for right in (left + 1)..graph.nodes.len() {
                let a = &graph.nodes[left];
                let b = &graph.nodes[right];
                let delta = positions[&a.id] - positions[&b.id];
                let distance = delta.length().max(1.0);
                let force = delta.normalized() * ((ideal_length * ideal_length) / distance);
                *forces.get_mut(&a.id).expect("node force must exist") += force;
                *forces.get_mut(&b.id).expect("node force must exist") += force * -1.0;
            }
        }

        for edge in &graph.edges {
            let delta = positions[&edge.target] - positions[&edge.source];
            let distance = delta.length().max(1.0);
            let force = delta.normalized() * ((distance * distance) / ideal_length) * 0.015;
            *forces
                .get_mut(&edge.source)
                .expect("edge source force must exist") += force;
            *forces
                .get_mut(&edge.target)
                .expect("edge target force must exist") += force * -1.0;
        }

        for node in &graph.nodes {
            let to_center = center - positions[&node.id];
            *forces.get_mut(&node.id).expect("center force must exist") += to_center * gravity;
        }

        for node in &graph.nodes {
            let id = &node.id;
            let displacement = forces[id];
            let length = displacement.length().max(1.0);
            let limited = displacement.normalized() * length.min(temperature);
            let next_position = (positions[id] + limited).clamp(width, height, padding);
            positions.insert(id.clone(), next_position);
        }

        let cooling = 1.0 - (step as f32 / iterations.max(1) as f32);
        temperature = (ideal_length * 0.08).max(temperature * 0.96 * cooling.max(0.35));
    }

    positions
}

fn seeded_unit(seed: &str) -> f32 {
    let hash = seed.bytes().fold(0u32, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u32)
    });
    (hash % 1000) as f32 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> GraphData {
        GraphData {
            nodes: vec![
                GraphNode {
                    id: "agent".into(),
                    label: "Agent Runtime".into(),
                    category: GraphNodeCategory::Service,
                    description: "执行自动化任务".into(),
                    details: "负责编排技能、任务和同步流程".into(),
                },
                GraphNode {
                    id: "pg".into(),
                    label: "Postgres".into(),
                    category: GraphNodeCategory::Database,
                    description: "知识元数据存储".into(),
                    details: "沉淀节点、标签和权限索引".into(),
                },
                GraphNode {
                    id: "skill".into(),
                    label: "Skill Catalog".into(),
                    category: GraphNodeCategory::Knowledge,
                    description: "可复用技能目录".into(),
                    details: "维护 SKILL.md 与引用关系".into(),
                },
                GraphNode {
                    id: "flow".into(),
                    label: "Publish Workflow".into(),
                    category: GraphNodeCategory::Workflow,
                    description: "发布和审计链路".into(),
                    details: "覆盖创建、同步、审核三个阶段".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "agent".into(),
                    target: "pg".into(),
                    kind: GraphEdgeKind::DependsOn,
                    label: None,
                },
                GraphEdge {
                    source: "agent".into(),
                    target: "skill".into(),
                    kind: GraphEdgeKind::Reads,
                    label: Some("读取目录".into()),
                },
                GraphEdge {
                    source: "flow".into(),
                    target: "agent".into(),
                    kind: GraphEdgeKind::Emits,
                    label: None,
                },
            ],
        }
    }

    #[test]
    fn filters_by_category() {
        let graph = sample_graph();
        let filtered = filter_graph(&graph, &BTreeSet::from([GraphNodeCategory::Service]), "", 1);

        assert_eq!(filtered.nodes.len(), 1);
        assert_eq!(filtered.nodes[0].id, "agent");
        assert!(filtered.edges.is_empty());
    }

    #[test]
    fn search_expands_one_hop_neighbors() {
        let graph = sample_graph();
        let filtered = filter_graph(&graph, &BTreeSet::new(), "catalog", 1);

        let ids: HashSet<_> = filtered.nodes.iter().map(|node| node.id.as_str()).collect();
        assert!(ids.contains("skill"));
        assert!(ids.contains("agent"));
        assert_eq!(filtered.edges.len(), 1);
    }

    #[test]
    fn layout_stays_inside_canvas() {
        let graph = sample_graph();
        let filtered = filter_graph(&graph, &BTreeSet::new(), "", 1);
        let positions = force_layout(&filtered, 960.0, 560.0, 80);

        assert_eq!(positions.len(), filtered.nodes.len());
        for position in positions.values() {
            assert!((56.0..=904.0).contains(&position.x));
            assert!((56.0..=504.0).contains(&position.y));
        }
    }
}
