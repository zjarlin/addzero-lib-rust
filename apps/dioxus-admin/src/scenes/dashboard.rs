use std::collections::{HashMap, HashSet};

use chrono::Local;
use dioxus::prelude::*;
use dioxus_components::{
    ContentHeader, Field, KeywordChips, ListItem, MetricRow, ResponsiveGrid, SidebarSection, Stack,
    Surface, SurfaceHeader, Textarea, Tone, WorkbenchButton,
};

use crate::{
    knowledge_catalog::{KNOWLEDGE_DOCS, KnowledgeDoc, total_bytes, total_sections},
    package_catalog::{PACKAGE_CHANNELS, package_assets},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DashboardLens {
    Notes,
    Software,
    Packages,
}

impl DashboardLens {
    const ALL: [Self; 3] = [Self::Notes, Self::Software, Self::Packages];

    fn label(self) -> &'static str {
        match self {
            Self::Notes => "笔记",
            Self::Software => "软件",
            Self::Packages => "安装包",
        }
    }

    fn root_id(self) -> &'static str {
        match self {
            Self::Notes => "lens-notes",
            Self::Software => "lens-software",
            Self::Packages => "lens-packages",
        }
    }

    fn entry_kind(self) -> KnowledgeEntryKind {
        match self {
            Self::Notes => KnowledgeEntryKind::Note,
            Self::Software => KnowledgeEntryKind::Software,
            Self::Packages => KnowledgeEntryKind::Package,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KnowledgeEntryKind {
    Note,
    Software,
    Package,
}

impl KnowledgeEntryKind {
    const ALL: [Self; 3] = [Self::Note, Self::Software, Self::Package];

    fn label(self) -> &'static str {
        match self {
            Self::Note => "笔记",
            Self::Software => "软件",
            Self::Package => "安装包",
        }
    }

    fn default_source(self) -> &'static str {
        match self {
            Self::Note => "阅读 / 对话 / 研究",
            Self::Software => "巡检 / 运行 / 集成",
            Self::Package => "下载 / 发布 / 归档",
        }
    }

    fn lens(self) -> DashboardLens {
        match self {
            Self::Note => DashboardLens::Notes,
            Self::Software => DashboardLens::Software,
            Self::Package => DashboardLens::Packages,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GraphNodeKind {
    Root,
    Cluster,
    Leaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GraphNodeItem {
    id: String,
    label: String,
    detail: String,
    meta: String,
    parent: Option<String>,
    kind: GraphNodeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GraphEdgeItem {
    source: String,
    target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GraphViewModel {
    root_id: String,
    root_label: String,
    nodes: Vec<GraphNodeItem>,
    edges: Vec<GraphEdgeItem>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GraphPoint {
    x: f32,
    y: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KnowledgeEntryRecord {
    title: String,
    body: String,
    source: String,
    tags: Vec<String>,
    kind: KnowledgeEntryKind,
    anchored_node: Option<String>,
    captured_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EntryFormState {
    title: String,
    source: String,
    body: String,
    tags: Vec<String>,
    kind: KnowledgeEntryKind,
}

impl EntryFormState {
    fn for_capture(kind: KnowledgeEntryKind) -> Self {
        Self {
            title: String::new(),
            source: kind.default_source().to_string(),
            body: String::new(),
            tags: vec![kind.label().to_string()],
            kind,
        }
    }

    fn for_node(kind: KnowledgeEntryKind, node: &GraphNodeItem) -> Self {
        Self {
            title: node.label.clone(),
            source: node.meta.clone(),
            body: node.detail.clone(),
            tags: vec![kind.label().to_string()],
            kind,
        }
    }

    fn cleaned_tags(&self) -> Vec<String> {
        self.tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeDraftShape {
    Cluster,
    Leaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EntryEditorMode {
    Capture,
    Create {
        parent_id: String,
        anchor_id: String,
        shape: NodeDraftShape,
    },
    Update {
        node_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NodeContextMenu {
    node_id: String,
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeleteNodeDialog {
    node_id: String,
}

#[derive(Clone)]
struct NoteCluster {
    label: &'static str,
    docs: Vec<&'static KnowledgeDoc>,
}

#[component]
pub fn Dashboard() -> Element {
    let active_lens = use_signal(|| DashboardLens::Notes);
    let mut search = use_signal(String::new);
    let focused_id = use_signal(|| DashboardLens::Notes.root_id().to_string());
    let graph_workspaces = use_signal(seed_graph_workspaces);
    let entry_log = use_signal(seed_entry_log);
    let entry_form = use_signal(|| EntryFormState::for_capture(DashboardLens::Notes.entry_kind()));
    let editor_mode = use_signal(|| EntryEditorMode::Capture);
    let context_menu = use_signal(|| None::<NodeContextMenu>);
    let delete_dialog = use_signal(|| None::<DeleteNodeDialog>);

    let lens = *active_lens.read();
    let graph = graph_workspaces
        .read()
        .get(&lens)
        .cloned()
        .unwrap_or_else(|| graph_for(lens));
    let filtered_graph = filter_graph(&graph, search.read().as_str());
    let positions = layout_graph(&filtered_graph, 960.0, 520.0);
    let focused = filtered_graph
        .nodes
        .iter()
        .find(|node| node.id == *focused_id.read())
        .cloned()
        .or_else(|| filtered_graph.nodes.first().cloned());
    let node_count = filtered_graph.nodes.len();
    let edge_count = filtered_graph.edges.len();
    let edge_items = filtered_graph
        .edges
        .iter()
        .filter_map(|edge| {
            let source = positions.get(edge.source.as_str())?;
            let target = positions.get(edge.target.as_str())?;
            Some((format!("{}-{}", edge.source, edge.target), *source, *target))
        })
        .collect::<Vec<_>>();
    let node_items = filtered_graph
        .nodes
        .iter()
        .filter_map(|node| {
            let position = positions.get(node.id.as_str())?;
            Some((
                node.id.clone(),
                node.label.clone(),
                node.meta.clone(),
                *position,
                graph_node_class(node, focused.as_ref()),
                graph_node_radius(node),
            ))
        })
        .collect::<Vec<_>>();
    let form = entry_form.read().clone();
    let mode = editor_mode.read().clone();
    let queue_preview = entry_log.read().iter().take(6).cloned().collect::<Vec<_>>();
    let menu_state = context_menu.read().clone();
    let delete_state = delete_dialog.read().clone();
    let menu_target = menu_state
        .as_ref()
        .and_then(|menu| graph_node(&graph, menu.node_id.as_str()))
        .cloned();
    let delete_target = delete_state
        .as_ref()
        .and_then(|dialog| graph_node(&graph, dialog.node_id.as_str()))
        .cloned();
    let editor_anchor = entry_anchor_copy(&graph, &mode);

    rsx! {
        ContentHeader {
            title: "知识图谱概览".to_string(),
            subtitle: "图谱浏览、节点操作和条目录入放在同一工作面。".to_string()
        }
        div { class: "lens-strip",
            for item in DashboardLens::ALL {
                WorkbenchButton {
                    class: "lens-pill".to_string(),
                    tone: if item == lens { Some(Tone::Accent) } else { None },
                    onclick: {
                        let mut active_lens = active_lens;
                        let mut search = search;
                        let mut focused_id = focused_id;
                        let mut entry_form = entry_form;
                        let mut editor_mode = editor_mode;
                        let mut context_menu = context_menu;
                        let mut delete_dialog = delete_dialog;
                        move |_| {
                            active_lens.set(item);
                            search.set(String::new());
                            focused_id.set(item.root_id().to_string());
                            entry_form.set(EntryFormState::for_capture(item.entry_kind()));
                            editor_mode.set(EntryEditorMode::Capture);
                            context_menu.set(None);
                            delete_dialog.set(None);
                        }
                    },
                    "{item.label()}"
                }
            }
        }
        Surface {
            SurfaceHeader {
                title: format!("{} 标签图谱", lens.label()),
                subtitle: match lens {
                    DashboardLens::Notes => "聚焦当前知识图谱。".to_string(),
                    DashboardLens::Software => "软件台账与关联节点。".to_string(),
                    DashboardLens::Packages => "安装包清单与依赖关系。".to_string(),
                },
                actions: rsx!(
                    div { class: "graph-toolbar",
                        input {
                            class: "graph-search",
                            r#type: "search",
                            value: "{search}",
                            placeholder: "搜索节点、专题、路径或关键词",
                            oninput: move |evt| search.set(evt.value())
                        }
                    }
                )
            }
            div { class: "graph-caption",
                span { class: "badge badge--fs", "当前图谱" }
                span { "{lens.label()}" }
                span { class: "graph-caption__dot", "•" }
                span { "{node_count} 节点 / {edge_count} 连线" }
            }
            if filtered_graph.nodes.is_empty() {
                div { class: "empty-state",
                    "没有匹配结果，调整搜索词后会自动回填图谱。"
                }
            } else {
                div { class: "graph-workbench",
                    div { class: "graph-canvas",
                        svg {
                            class: "graph-canvas__svg",
                            view_box: "0 0 960 520",
                            preserve_aspect_ratio: "xMidYMid meet",
                            for (key, source, target) in edge_items {
                                line {
                                    key: "{key}",
                                    class: "graph-edge",
                                    x1: "{source.x}",
                                    y1: "{source.y}",
                                    x2: "{target.x}",
                                    y2: "{target.y}",
                                }
                            }
                            for (id, label, meta, position, class, radius) in node_items {
                                g {
                                    key: "{id}",
                                    class: "{class}",
                                    onclick: {
                                        let mut focused_id = focused_id;
                                        let id = id.clone();
                                        move |_| focused_id.set(id.clone())
                                    },
                                    oncontextmenu: {
                                        let mut context_menu = context_menu;
                                        let mut focused_id = focused_id;
                                        let id = id.clone();
                                        move |evt| {
                                            evt.prevent_default();
                                            let point = evt.client_coordinates();
                                            focused_id.set(id.clone());
                                            context_menu.set(Some(NodeContextMenu {
                                                node_id: id.clone(),
                                                x: point.x.round() as i32,
                                                y: point.y.round() as i32,
                                            }));
                                        }
                                    },
                                    circle {
                                        class: "graph-node__circle",
                                        cx: "{position.x}",
                                        cy: "{position.y}",
                                        r: "{radius}"
                                    }
                                    text {
                                        class: "graph-node__label",
                                        x: "{position.x}",
                                        y: "{position.y - 2.0}",
                                        text_anchor: "middle",
                                        dominant_baseline: "central",
                                        "{label}"
                                    }
                                    text {
                                        class: "graph-node__meta",
                                        x: "{position.x}",
                                        y: "{position.y + radius + 16.0}",
                                        text_anchor: "middle",
                                        "{meta}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "graph-sidepanel",
                        div { class: "graph-detail",
                            if let Some(node) = focused.clone() {
                                div { class: "graph-detail__eyebrow",
                                    span { class: "badge badge--fs", "{lens.label()}" }
                                    span { class: "graph-detail__kind", "{graph_kind_label(&node.kind)}" }
                                }
                                h3 { class: "graph-detail__title", "{node.label}" }
                                p { class: "graph-detail__copy", "{node.detail}" }
                                p { class: "graph-detail__copy graph-detail__copy--muted", "{node.meta}" }
                                if let Some(parent) = &node.parent {
                                    p { class: "graph-detail__copy graph-detail__copy--muted",
                                        "上级节点："
                                        "{graph_parent_label(&filtered_graph, parent)}"
                                    }
                                }
                                div { class: "callout callout--info",
                                    "节点上右键即可查看、新增、编辑或删除；根节点只允许新增分组，不允许直接改删。"
                                }
                            }
                        }
                        div { class: "entry-dock",
                            div { class: "entry-dock__header",
                                h3 { class: "graph-detail__title", "{entry_editor_title(&mode)}" }
                                p { class: "graph-detail__copy graph-detail__copy--muted", "{entry_editor_subtitle(&mode)}" }
                            }
                            if let Some(anchor) = editor_anchor {
                                div { class: "callout callout--info", "{anchor}" }
                            } else {
                                div { class: "callout callout--info",
                                    "默认会把条目挂到当前图谱的分组节点下；如果要精确挂载，请先右键目标节点再新增。"
                                }
                            }
                            if matches!(mode, EntryEditorMode::Capture) {
                                div { class: "entry-kind-strip",
                                    for kind in KnowledgeEntryKind::ALL {
                                        WorkbenchButton {
                                            class: "entry-kind-pill".to_string(),
                                            tone: if kind == form.kind { Some(Tone::Accent) } else { None },
                                            onclick: {
                                                let mut entry_form = entry_form;
                                                move |_| {
                                                    let mut next = entry_form.read().clone();
                                                    next.kind = kind;
                                                    if next.tags.is_empty() {
                                                        next.tags = vec![kind.label().to_string()];
                                                    }
                                                    if next.source.trim().is_empty() {
                                                        next.source = kind.default_source().to_string();
                                                    }
                                                    entry_form.set(next);
                                                }
                                            },
                                            "{kind.label()}"
                                        }
                                    }
                                }
                            } else {
                                div { class: "entry-kind-strip",
                                    span { class: "badge badge--fs", "当前图谱" }
                                    span { class: "badge", "{lens.label()}" }
                                }
                            }
                            ResponsiveGrid { columns: 2,
                                Field {
                                    label: "标题".to_string(),
                                    value: form.title.clone(),
                                    placeholder: Some(match mode {
                                        EntryEditorMode::Create { shape: NodeDraftShape::Cluster, .. } => "例如：运行时依赖".to_string(),
                                        _ => "例如：ownership 讲解适合补图示".to_string(),
                                    }),
                                    on_input: {
                                        let mut entry_form = entry_form;
                                        move |value| {
                                            let mut next = entry_form.read().clone();
                                            next.title = value;
                                            entry_form.set(next);
                                        }
                                    }
                                }
                                Field {
                                    label: "来源 / 元信息".to_string(),
                                    value: form.source.clone(),
                                    placeholder: Some("会议 / 对话 / 文章 / 代码 / 版本信息".to_string()),
                                    on_input: {
                                        let mut entry_form = entry_form;
                                        move |value| {
                                            let mut next = entry_form.read().clone();
                                            next.source = value;
                                            entry_form.set(next);
                                        }
                                    }
                                }
                            }
                            KeywordChips {
                                value: form.tags.clone(),
                                placeholder: Some("补充标签、平台或专题名".to_string()),
                                on_change: {
                                    let mut entry_form = entry_form;
                                    move |next| {
                                        let mut form = entry_form.read().clone();
                                        form.tags = next;
                                        entry_form.set(form);
                                    }
                                }
                            }
                            Textarea {
                                label: entry_body_label(&mode).to_string(),
                                value: form.body.clone(),
                                rows: Some(6),
                                placeholder: Some("记录说明、关系、待验证假设，或节点的补充上下文。".to_string()),
                                on_input: {
                                    let mut entry_form = entry_form;
                                    move |value| {
                                        let mut next = entry_form.read().clone();
                                        next.body = value;
                                        entry_form.set(next);
                                    }
                                }
                            }
                            div { class: "entry-actions",
                                WorkbenchButton {
                                    class: "action-button action-button--primary".to_string(),
                                    onclick: {
                                        let mut graph_workspaces = graph_workspaces;
                                        let mut entry_log = entry_log;
                                        let mut entry_form = entry_form;
                                        let mut editor_mode = editor_mode;
                                        let mut focused_id = focused_id;
                                        let mut search = search;
                                        let current_lens = lens;
                                        move |_| {
                                            let form = entry_form.read().clone();
                                            let mode = editor_mode.read().clone();
                                            let title = form.title.trim().to_string();
                                            let body = form.body.trim().to_string();
                                            let tags = form.cleaned_tags();
                                            let source = form.source.trim().to_string();

                                            if title.is_empty() && body.is_empty() {
                                                return;
                                            }

                                            match mode {
                                                EntryEditorMode::Capture => {
                                                    let target_lens = form.kind.lens();
                                                    let preferred_focus = if target_lens == current_lens {
                                                        Some(focused_id.read().clone())
                                                    } else {
                                                        None
                                                    };
                                                    let mut created_node_id = None;
                                                    let mut anchored_label = None;
                                                    graph_workspaces.with_mut(|workspaces| {
                                                        let graph = workspaces
                                                            .entry(target_lens)
                                                            .or_insert_with(|| graph_for(target_lens));
                                                        anchored_label = preferred_focus
                                                            .as_deref()
                                                            .and_then(|node_id| graph_node(graph, node_id))
                                                            .map(|node| node.label.clone());
                                                        let parent_id =
                                                            default_capture_parent(graph, preferred_focus.as_deref());
                                                        created_node_id = insert_graph_node(
                                                            graph,
                                                            &parent_id,
                                                            NodeDraftShape::Leaf,
                                                            &title,
                                                            &body,
                                                            &source,
                                                            &tags,
                                                        );
                                                    });
                                                    entry_log.with_mut(|queue| {
                                                        queue.insert(
                                                            0,
                                                            KnowledgeEntryRecord {
                                                                title: fallback_leaf_label(&title),
                                                                body: body.clone(),
                                                                source: if source.is_empty() {
                                                                    "未注明来源".to_string()
                                                                } else {
                                                                    source.clone()
                                                                },
                                                                tags: if tags.is_empty() {
                                                                    vec![form.kind.label().to_string()]
                                                                } else {
                                                                    tags.clone()
                                                                },
                                                                kind: form.kind,
                                                                anchored_node: anchored_label,
                                                                captured_at: Local::now()
                                                                    .format("%H:%M")
                                                                    .to_string(),
                                                            },
                                                        );
                                                        if queue.len() > 8 {
                                                            queue.truncate(8);
                                                        }
                                                    });
                                                    if target_lens == current_lens {
                                                        if let Some(node_id) = created_node_id {
                                                            search.set(String::new());
                                                            focused_id.set(node_id);
                                                        }
                                                    }
                                                    entry_form.set(EntryFormState::for_capture(current_lens.entry_kind()));
                                                }
                                                EntryEditorMode::Create {
                                                    parent_id,
                                                    anchor_id,
                                                    shape,
                                                } => {
                                                    let mut created_node_id = None;
                                                    graph_workspaces.with_mut(|workspaces| {
                                                        let graph = workspaces
                                                            .entry(current_lens)
                                                            .or_insert_with(|| graph_for(current_lens));
                                                        created_node_id = insert_graph_node(
                                                            graph,
                                                            &parent_id,
                                                            shape,
                                                            &title,
                                                            &body,
                                                            &source,
                                                            &tags,
                                                        );
                                                    });
                                                    if shape == NodeDraftShape::Leaf {
                                                        let anchor_label = graph_node(&graph, &anchor_id)
                                                            .map(|node| node.label.clone());
                                                        entry_log.with_mut(|queue| {
                                                            queue.insert(
                                                                0,
                                                                KnowledgeEntryRecord {
                                                                    title: fallback_leaf_label(&title),
                                                                    body: body.clone(),
                                                                    source: if source.is_empty() {
                                                                        "未注明来源".to_string()
                                                                    } else {
                                                                        source.clone()
                                                                    },
                                                                    tags: if tags.is_empty() {
                                                                        vec![current_lens.label().to_string()]
                                                                    } else {
                                                                        tags.clone()
                                                                    },
                                                                    kind: current_lens.entry_kind(),
                                                                    anchored_node: anchor_label,
                                                                    captured_at: Local::now()
                                                                        .format("%H:%M")
                                                                        .to_string(),
                                                                },
                                                            );
                                                            if queue.len() > 8 {
                                                                queue.truncate(8);
                                                            }
                                                        });
                                                    }
                                                    if let Some(node_id) = created_node_id {
                                                        search.set(String::new());
                                                        focused_id.set(node_id);
                                                    }
                                                    editor_mode.set(EntryEditorMode::Capture);
                                                    entry_form.set(EntryFormState::for_capture(current_lens.entry_kind()));
                                                }
                                                EntryEditorMode::Update { node_id } => {
                                                    graph_workspaces.with_mut(|workspaces| {
                                                        if let Some(graph) = workspaces.get_mut(&current_lens) {
                                                            update_graph_node(graph, &node_id, &title, &body, &source, &tags);
                                                        }
                                                    });
                                                    search.set(String::new());
                                                    focused_id.set(node_id);
                                                    editor_mode.set(EntryEditorMode::Capture);
                                                    entry_form.set(EntryFormState::for_capture(current_lens.entry_kind()));
                                                }
                                            }
                                        }
                                    },
                                    "{entry_submit_label(&mode)}"
                                }
                                if !matches!(mode, EntryEditorMode::Capture) {
                                    WorkbenchButton {
                                        class: "action-button".to_string(),
                                        onclick: {
                                            let mut editor_mode = editor_mode;
                                            let mut entry_form = entry_form;
                                            move |_| {
                                                editor_mode.set(EntryEditorMode::Capture);
                                                entry_form.set(EntryFormState::for_capture(lens.entry_kind()));
                                            }
                                        },
                                        "取消操作"
                                    }
                                }
                            }
                            span { class: "entry-actions__hint",
                                "当前仍是前端本地图谱态，用来确认命名、多态模型和节点交互。"
                            }
                            div { class: "entry-log",
                                div { class: "entry-log__title", "最近录入" }
                                Stack {
                                    for item in queue_preview {
                                        ListItem {
                                            title: format!("{} · {}", item.kind.label(), item.title),
                                            detail: truncate_copy(&item.body, 88),
                                            meta: if let Some(anchor) = &item.anchored_node {
                                                format!("{} · {} · {}", item.captured_at, item.source, anchor)
                                            } else {
                                                format!("{} · {}", item.captured_at, item.source)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if let (Some(menu), Some(node)) = (menu_state, menu_target) {
            div {
                class: "graph-context-shell",
                onclick: {
                    let mut context_menu = context_menu;
                    move |_| context_menu.set(None)
                },
                div {
                    class: "graph-context-menu",
                    style: "top: {menu.y}px; left: {menu.x}px;",
                    onclick: move |evt| evt.stop_propagation(),
                    button {
                        r#type: "button",
                        class: "graph-context-menu__item",
                        onclick: {
                            let mut focused_id = focused_id;
                            let mut context_menu = context_menu;
                            let node_id = node.id.clone();
                            move |_| {
                                focused_id.set(node_id.clone());
                                context_menu.set(None);
                            }
                        },
                        "查看节点"
                    }
                    if let Some((create_label, create_mode)) = menu_create_action(&node) {
                        button {
                            r#type: "button",
                            class: "graph-context-menu__item",
                            onclick: {
                                let mut editor_mode = editor_mode;
                                let mut entry_form = entry_form;
                                let mut context_menu = context_menu;
                                let lens_kind = lens.entry_kind();
                                move |_| {
                                    editor_mode.set(create_mode.clone());
                                    entry_form.set(EntryFormState::for_capture(lens_kind));
                                    context_menu.set(None);
                                }
                            },
                            "{create_label}"
                        }
                    }
                    if can_edit_node(&node) {
                        button {
                            r#type: "button",
                            class: "graph-context-menu__item",
                            onclick: {
                                let mut editor_mode = editor_mode;
                                let mut entry_form = entry_form;
                                let mut context_menu = context_menu;
                                let lens_kind = lens.entry_kind();
                                let node = node.clone();
                                move |_| {
                                    editor_mode.set(EntryEditorMode::Update {
                                        node_id: node.id.clone(),
                                    });
                                    entry_form.set(EntryFormState::for_node(lens_kind, &node));
                                    context_menu.set(None);
                                }
                            },
                            "编辑节点"
                        }
                    }
                    if can_delete_node(&node) {
                        button {
                            r#type: "button",
                            class: "graph-context-menu__item graph-context-menu__item--danger",
                            onclick: {
                                let mut delete_dialog = delete_dialog;
                                let mut context_menu = context_menu;
                                let node_id = node.id.clone();
                                move |_| {
                                    delete_dialog.set(Some(DeleteNodeDialog { node_id: node_id.clone() }));
                                    context_menu.set(None);
                                }
                            },
                            "删除节点"
                        }
                    }
                }
            }
        }
        if let Some(node) = delete_target {
            div { class: "dialog",
                div {
                    class: "dialog__backdrop",
                    onclick: {
                        let mut delete_dialog = delete_dialog;
                        move |_| delete_dialog.set(None)
                    }
                }
                div { class: "dialog__panel",
                    h3 { class: "dialog__title", "删除节点" }
                    p { class: "dialog__message",
                        "将删除 “{node.label}” 及其下挂子节点。当前仍是前端本地图谱态，但删除会立即反映到当前工作面。"
                    }
                    div { class: "dialog__actions",
                        button {
                            r#type: "button",
                            class: "dialog__button",
                            onclick: {
                                let mut delete_dialog = delete_dialog;
                                move |_| delete_dialog.set(None)
                            },
                            "取消"
                        }
                        button {
                            r#type: "button",
                            class: "dialog__button dialog__button--danger",
                            onclick: {
                                let mut graph_workspaces = graph_workspaces;
                                let mut focused_id = focused_id;
                                let mut delete_dialog = delete_dialog;
                                let mut editor_mode = editor_mode;
                                let mut entry_form = entry_form;
                                let node_id = node.id.clone();
                                let current_lens = lens;
                                move |_| {
                                    graph_workspaces.with_mut(|workspaces| {
                                        if let Some(graph) = workspaces.get_mut(&current_lens) {
                                            delete_graph_subtree(graph, &node_id);
                                            focused_id.set(graph.root_id.clone());
                                        }
                                    });
                                    delete_dialog.set(None);
                                    editor_mode.set(EntryEditorMode::Capture);
                                    entry_form.set(EntryFormState::for_capture(current_lens.entry_kind()));
                                }
                            },
                            "删除节点"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Audit() -> Element {
    rsx! {
        ContentHeader {
            title: "审计日志".to_string(),
            subtitle: "右侧上下文栏继续承担辅助理解，不抢主内容权重。".to_string()
        }
        Surface {
            SurfaceHeader {
                title: "最近日志".to_string(),
                subtitle: "先有时间线，再接过滤器和详情抽屉。".to_string()
            }
            Stack {
                ListItem { title: "09:12 审计任务结束".to_string(), detail: "策略审计通过，没有新增风险项".to_string(), meta: "system".to_string() }
                ListItem { title: "08:41 权限模板变更".to_string(), detail: "P-09 删除了一条遗留白名单".to_string(), meta: "Chen".to_string() }
                ListItem { title: "昨天 18:20 发布完成".to_string(), detail: "release-0426 已部署到 production".to_string(), meta: "Luna".to_string() }
            }
        }
    }
}

#[component]
pub fn DashboardContext() -> Element {
    let note_docs = note_docs();
    let note_section_total = note_docs.iter().map(|doc| doc.section_count).sum::<usize>();

    rsx! {
        SidebarSection { label: "当前默认".to_string(),
            Stack {
                MetricRow { label: "标签".to_string(), value: "笔记".to_string(), tone: Tone::Accent }
                MetricRow { label: "已纳入文档".to_string(), value: note_docs.len().to_string() }
                MetricRow { label: "章节小节".to_string(), value: note_section_total.to_string() }
            }
        }
        SidebarSection { label: "图谱基线".to_string(),
            Stack {
                for cluster in note_clusters() {
                    MetricRow { label: cluster.label.to_string(), value: cluster.docs.len().to_string() }
                }
            }
        }
        SidebarSection { label: "条目录入".to_string(),
            div { class: "callout callout--info",
                "首页上的条目录入当前先做本地图谱暂存，下一步可以直接接入知识库写入动作。"
            }
        }
        SidebarSection { label: "体量".to_string(),
            Stack {
                MetricRow { label: "Rust 摘录".to_string(), value: format_bytes(note_docs.iter().map(|doc| doc.bytes).sum()) }
                MetricRow { label: "全部知识".to_string(), value: format_bytes(total_bytes()) }
                MetricRow { label: "全库小节".to_string(), value: total_sections().to_string() }
            }
        }
    }
}

#[component]
pub fn DefaultContext() -> Element {
    rsx! {
        SidebarSection { label: "概览".to_string(),
            Stack {
                MetricRow { label: "成功率".to_string(), value: "99.2%".to_string(), tone: Tone::Positive }
                MetricRow { label: "告警数".to_string(), value: "03".to_string(), tone: Tone::Warning }
                MetricRow { label: "挂起变更".to_string(), value: "07".to_string() }
            }
        }
        SidebarSection { label: "最近动作".to_string(),
            Stack {
                ListItem {
                    title: "规则 A-17 已提交".to_string(),
                    detail: "Luna 更新了命中条件".to_string(),
                    meta: "2 分钟前".to_string()
                }
                ListItem {
                    title: "执行器扩容完成".to_string(),
                    detail: "队列延迟恢复到基线".to_string(),
                    meta: "18 分钟前".to_string()
                }
                ListItem {
                    title: "审计任务结束".to_string(),
                    detail: "未发现新增风险项".to_string(),
                    meta: "今天 09:12".to_string()
                }
            }
        }
    }
}

fn graph_for(lens: DashboardLens) -> GraphViewModel {
    match lens {
        DashboardLens::Notes => notes_graph(),
        DashboardLens::Software => software_graph(),
        DashboardLens::Packages => packages_graph(),
    }
}

fn notes_graph() -> GraphViewModel {
    let root_id = DashboardLens::Notes.root_id().to_string();
    let mut nodes = vec![GraphNodeItem {
        id: root_id.clone(),
        label: "笔记".to_string(),
        detail: "当前默认标签域，聚合已纳入后台的 Rust 学习资料，用于图谱化浏览与后续知识入湖。"
            .to_string(),
        meta: format!("{} 篇文档", note_docs().len()),
        parent: None,
        kind: GraphNodeKind::Root,
    }];
    let mut edges = Vec::new();

    for cluster in note_clusters() {
        let cluster_id = format!("notes-{}", cluster.label);
        nodes.push(GraphNodeItem {
            id: cluster_id.clone(),
            label: cluster.label.to_string(),
            detail: format!("按学习阶段聚合，当前收录 {} 篇。", cluster.docs.len()),
            meta: format!("{} 篇", cluster.docs.len()),
            parent: Some(root_id.clone()),
            kind: GraphNodeKind::Cluster,
        });
        edges.push(GraphEdgeItem {
            source: root_id.clone(),
            target: cluster_id.clone(),
        });

        for doc in cluster.docs {
            nodes.push(note_doc_node(doc, &cluster_id));
            edges.push(GraphEdgeItem {
                source: cluster_id.clone(),
                target: doc.slug.to_string(),
            });
        }
    }

    GraphViewModel {
        root_id,
        root_label: "笔记".to_string(),
        nodes,
        edges,
    }
}

fn note_doc_node(doc: &KnowledgeDoc, parent: &str) -> GraphNodeItem {
    let chapter_label = note_doc_chapter_label(doc);

    GraphNodeItem {
        id: doc.slug.to_string(),
        label: doc.title.to_string(),
        detail: doc.preview.to_string(),
        meta: format!(
            "{} · {} 节 · {}",
            chapter_label,
            doc.section_count,
            format_bytes(doc.bytes)
        ),
        parent: Some(parent.to_string()),
        kind: GraphNodeKind::Leaf,
    }
}

fn software_graph() -> GraphViewModel {
    graph_from_seed(
        DashboardLens::Software,
        "软件",
        "软件资产围绕创作、基础设施和发布链路组织。",
        vec![
            (
                "创作工具",
                vec![
                    ("Cursor", "代理协作与代码编写主入口。", "IDE / Agent"),
                    ("Rust Analyzer", "语言服务与跳转分析。", "LSP"),
                ],
            ),
            (
                "本地基础设施",
                vec![
                    ("Docker Desktop", "容器编排、网络与镜像运行。", "Runtime"),
                    ("cloudflared", "本地服务暴露和隧道入口。", "Ingress"),
                ],
            ),
            (
                "发布链路",
                vec![
                    ("GitHub", "版本追踪、评审与发布记录。", "Repository"),
                    ("Cargo", "构建、测试与依赖管理。", "Build"),
                ],
            ),
        ],
    )
}

fn packages_graph() -> GraphViewModel {
    graph_from_seed(
        DashboardLens::Packages,
        "安装包",
        "先把安装包资产纳入统一目录，再补版本追踪、校验与分发链路。",
        PACKAGE_CHANNELS
            .iter()
            .map(|channel| {
                (
                    channel.title,
                    package_assets(channel.slug)
                        .map(|asset| (asset.software_title, asset.note, asset.platform))
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    )
}


/// (label, detail, meta) tuple for graph cluster items.
type ClusterItem<'a> = (&'a str, &'a str, &'a str);

fn graph_from_seed(
    lens: DashboardLens,
    root_label: &str,
    root_detail: &str,
    seed: Vec<(&str, Vec<ClusterItem<'_>>)>,
) -> GraphViewModel {
    let root_id = lens.root_id().to_string();
    let mut nodes = vec![GraphNodeItem {
        id: root_id.clone(),
        label: root_label.to_string(),
        detail: root_detail.to_string(),
        meta: format!("{} 组", seed.len()),
        parent: None,
        kind: GraphNodeKind::Root,
    }];
    let mut edges = Vec::new();

    for (cluster, items) in seed {
        let cluster_id = format!("{}-{}", root_id, cluster);
        nodes.push(GraphNodeItem {
            id: cluster_id.clone(),
            label: cluster.to_string(),
            detail: format!("{cluster} 下挂 {} 个节点。", items.len()),
            meta: format!("{} 项", items.len()),
            parent: Some(root_id.clone()),
            kind: GraphNodeKind::Cluster,
        });
        edges.push(GraphEdgeItem {
            source: root_id.clone(),
            target: cluster_id.clone(),
        });

        for (label, detail, meta) in items {
            let node_id = format!("{}-{}", cluster_id, label);
            nodes.push(GraphNodeItem {
                id: node_id.clone(),
                label: label.to_string(),
                detail: detail.to_string(),
                meta: meta.to_string(),
                parent: Some(cluster_id.clone()),
                kind: GraphNodeKind::Leaf,
            });
            edges.push(GraphEdgeItem {
                source: cluster_id.clone(),
                target: node_id,
            });
        }
    }

    GraphViewModel {
        root_id,
        root_label: root_label.to_string(),
        nodes,
        edges,
    }
}

fn filter_graph(graph: &GraphViewModel, search: &str) -> GraphViewModel {
    let query = search.trim().to_lowercase();
    if query.is_empty() {
        return graph.clone();
    }

    let by_id: HashMap<&str, &GraphNodeItem> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let children = child_map(&graph.nodes);
    let mut included = HashSet::from([graph.root_id.clone()]);

    for node in &graph.nodes {
        let matches = node.label.to_lowercase().contains(&query)
            || node.detail.to_lowercase().contains(&query)
            || node.meta.to_lowercase().contains(&query);
        if matches {
            included.insert(node.id.clone());
            include_ancestors(node, &by_id, &mut included);
            include_descendants(node.id.as_str(), &children, &mut included);
        }
    }

    let nodes = graph
        .nodes
        .iter()
        .filter(|node| included.contains(node.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let visible: HashSet<&str> = nodes.iter().map(|node| node.id.as_str()).collect();
    let edges = graph
        .edges
        .iter()
        .filter(|edge| {
            visible.contains(edge.source.as_str()) && visible.contains(edge.target.as_str())
        })
        .cloned()
        .collect();

    GraphViewModel {
        root_id: graph.root_id.clone(),
        root_label: graph.root_label.clone(),
        nodes,
        edges,
    }
}

fn child_map(nodes: &[GraphNodeItem]) -> HashMap<&str, Vec<&str>> {
    let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = &node.parent {
            map.entry(parent.as_str())
                .or_default()
                .push(node.id.as_str());
        }
    }
    map
}

fn include_ancestors(
    node: &GraphNodeItem,
    by_id: &HashMap<&str, &GraphNodeItem>,
    included: &mut HashSet<String>,
) {
    let mut current = node.parent.as_deref();
    while let Some(parent) = current {
        if !included.insert(parent.to_string()) {
            break;
        }
        current = by_id.get(parent).and_then(|item| item.parent.as_deref());
    }
}

fn include_descendants(
    node_id: &str,
    children: &HashMap<&str, Vec<&str>>,
    included: &mut HashSet<String>,
) {
    if let Some(next) = children.get(node_id) {
        for child in next {
            if included.insert((*child).to_string()) {
                include_descendants(child, children, included);
            }
        }
    }
}

fn layout_graph(graph: &GraphViewModel, width: f32, height: f32) -> HashMap<String, GraphPoint> {
    let center = GraphPoint {
        x: width * 0.5,
        y: height * 0.48,
    };
    let mut positions = HashMap::new();
    positions.insert(graph.root_id.clone(), center);

    let clusters = graph
        .nodes
        .iter()
        .filter(|node| node.parent.as_deref() == Some(graph.root_id.as_str()))
        .collect::<Vec<_>>();
    let leaves = graph
        .nodes
        .iter()
        .filter_map(|node| node.parent.as_ref().map(|parent| (parent.clone(), node)))
        .fold(
            HashMap::<String, Vec<&GraphNodeItem>>::new(),
            |mut acc, (parent, node)| {
                if parent != graph.root_id {
                    acc.entry(parent).or_default().push(node);
                }
                acc
            },
        );

    for (index, cluster) in clusters.iter().enumerate() {
        let angle = std::f32::consts::TAU * index as f32 / clusters.len().max(1) as f32
            - std::f32::consts::FRAC_PI_2;
        let cluster_point = GraphPoint {
            x: center.x + angle.cos() * 240.0,
            y: center.y + angle.sin() * 155.0,
        };
        positions.insert(cluster.id.clone(), cluster_point);

        if let Some(group) = leaves.get(cluster.id.as_str()) {
            for (leaf_index, leaf) in group.iter().enumerate() {
                let spread = std::f32::consts::TAU * leaf_index as f32 / group.len().max(1) as f32;
                let leaf_angle = if group.len() == 1 {
                    angle
                } else {
                    angle + spread * 0.68 - 0.68
                };
                positions.insert(
                    leaf.id.clone(),
                    GraphPoint {
                        x: cluster_point.x + leaf_angle.cos() * 114.0,
                        y: cluster_point.y + leaf_angle.sin() * 88.0,
                    },
                );
            }
        }
    }

    positions
}

fn graph_node_radius(node: &GraphNodeItem) -> f32 {
    match node.kind {
        GraphNodeKind::Root => 36.0,
        GraphNodeKind::Cluster => 24.0,
        GraphNodeKind::Leaf => 18.0,
    }
}

fn graph_node_class(node: &GraphNodeItem, focused: Option<&GraphNodeItem>) -> &'static str {
    let is_focused = focused.is_some_and(|current| current.id == node.id);
    match (node.kind.clone(), is_focused) {
        (GraphNodeKind::Root, true) => "graph-node graph-node--root graph-node--focused",
        (GraphNodeKind::Root, false) => "graph-node graph-node--root",
        (GraphNodeKind::Cluster, true) => "graph-node graph-node--cluster graph-node--focused",
        (GraphNodeKind::Cluster, false) => "graph-node graph-node--cluster",
        (GraphNodeKind::Leaf, true) => "graph-node graph-node--leaf graph-node--focused",
        (GraphNodeKind::Leaf, false) => "graph-node graph-node--leaf",
    }
}

fn graph_kind_label(kind: &GraphNodeKind) -> &'static str {
    match kind {
        GraphNodeKind::Root => "标签域",
        GraphNodeKind::Cluster => "专题簇",
        GraphNodeKind::Leaf => "知识节点",
    }
}

fn graph_parent_label(graph: &GraphViewModel, parent: &str) -> String {
    graph
        .nodes
        .iter()
        .find(|node| node.id == parent)
        .map(|node| node.label.clone())
        .unwrap_or_else(|| parent.to_string())
}

fn seed_graph_workspaces() -> HashMap<DashboardLens, GraphViewModel> {
    HashMap::from([
        (DashboardLens::Notes, notes_graph()),
        (DashboardLens::Software, software_graph()),
        (DashboardLens::Packages, packages_graph()),
    ])
}

fn seed_entry_log() -> Vec<KnowledgeEntryRecord> {
    vec![
        KnowledgeEntryRecord {
            title: "ownership 章节适合补图示".to_string(),
            body: "把 move / borrow / mutable borrow 的冲突关系整理成一张图，更适合知识图谱里的概念节点。".to_string(),
            source: "阅读 Rust 章节".to_string(),
            tags: vec!["笔记".to_string(), "所有权".to_string()],
            kind: KnowledgeEntryKind::Note,
            anchored_node: Some("语言核心".to_string()),
            captured_at: "09:18".to_string(),
        },
        KnowledgeEntryRecord {
            title: "cloudflared 可挂到软件台账".to_string(),
            body: "和 tunnel 相关安装包一起展示时，软件节点与安装包节点需要互相可跳转。".to_string(),
            source: "运维梳理".to_string(),
            tags: vec!["软件".to_string(), "安装包".to_string()],
            kind: KnowledgeEntryKind::Software,
            anchored_node: Some("本地基础设施".to_string()),
            captured_at: "昨天".to_string(),
        },
    ]
}

fn entry_editor_title(mode: &EntryEditorMode) -> &'static str {
    match mode {
        EntryEditorMode::Capture => "条目录入",
        EntryEditorMode::Create {
            shape: NodeDraftShape::Cluster,
            ..
        } => "新增分组节点",
        EntryEditorMode::Create {
            shape: NodeDraftShape::Leaf,
            ..
        } => "新增条目节点",
        EntryEditorMode::Update { .. } => "编辑节点",
    }
}

fn entry_editor_subtitle(mode: &EntryEditorMode) -> &'static str {
    match mode {
        EntryEditorMode::Capture => "统一录入笔记、软件、安装包三类条目，当前标签只作为默认值。",
        EntryEditorMode::Create {
            shape: NodeDraftShape::Cluster,
            ..
        } => "会直接挂到当前图谱根节点下，用于补充分组或专题簇。",
        EntryEditorMode::Create {
            shape: NodeDraftShape::Leaf,
            ..
        } => "会挂到目标分组下，并立刻在当前图谱里可见。",
        EntryEditorMode::Update { .. } => "修改标题、说明和元信息，不切换出当前图谱上下文。",
    }
}

fn entry_body_label(mode: &EntryEditorMode) -> &'static str {
    match mode {
        EntryEditorMode::Capture => "条目说明",
        _ => "节点说明",
    }
}

fn entry_submit_label(mode: &EntryEditorMode) -> &'static str {
    match mode {
        EntryEditorMode::Capture => "录入条目",
        EntryEditorMode::Create { .. } => "新增节点",
        EntryEditorMode::Update { .. } => "保存修改",
    }
}

fn entry_anchor_copy(graph: &GraphViewModel, mode: &EntryEditorMode) -> Option<String> {
    match mode {
        EntryEditorMode::Capture => None,
        EntryEditorMode::Create {
            anchor_id, shape, ..
        } => graph_node(graph, anchor_id).map(|node| match shape {
            NodeDraftShape::Cluster => format!("新分组会挂在“{}”根节点下。", node.label),
            NodeDraftShape::Leaf if node.kind == GraphNodeKind::Leaf => {
                format!("新节点会与“{}”并列，保持同一父分组。", node.label)
            }
            NodeDraftShape::Leaf => format!("新节点会挂在“{}”下。", node.label),
        }),
        EntryEditorMode::Update { node_id } => {
            graph_node(graph, node_id).map(|node| format!("正在编辑“{}”。", node.label))
        }
    }
}

fn menu_create_action(node: &GraphNodeItem) -> Option<(&'static str, EntryEditorMode)> {
    match node.kind {
        GraphNodeKind::Root => Some((
            "新增分组节点",
            EntryEditorMode::Create {
                parent_id: node.id.clone(),
                anchor_id: node.id.clone(),
                shape: NodeDraftShape::Cluster,
            },
        )),
        GraphNodeKind::Cluster => Some((
            "新增子节点",
            EntryEditorMode::Create {
                parent_id: node.id.clone(),
                anchor_id: node.id.clone(),
                shape: NodeDraftShape::Leaf,
            },
        )),
        GraphNodeKind::Leaf => node.parent.clone().map(|parent_id| {
            (
                "新增同级节点",
                EntryEditorMode::Create {
                    parent_id,
                    anchor_id: node.id.clone(),
                    shape: NodeDraftShape::Leaf,
                },
            )
        }),
    }
}

fn can_edit_node(node: &GraphNodeItem) -> bool {
    !matches!(node.kind, GraphNodeKind::Root)
}

fn can_delete_node(node: &GraphNodeItem) -> bool {
    !matches!(node.kind, GraphNodeKind::Root)
}

fn graph_node<'a>(graph: &'a GraphViewModel, node_id: &str) -> Option<&'a GraphNodeItem> {
    graph.nodes.iter().find(|node| node.id == node_id)
}

fn default_capture_parent(graph: &GraphViewModel, preferred_focus: Option<&str>) -> String {
    preferred_focus
        .and_then(|node_id| graph_node(graph, node_id))
        .and_then(|node| match node.kind {
            GraphNodeKind::Root => first_cluster_id(graph),
            GraphNodeKind::Cluster => Some(node.id.clone()),
            GraphNodeKind::Leaf => node.parent.clone(),
        })
        .or_else(|| first_cluster_id(graph))
        .unwrap_or_else(|| graph.root_id.clone())
}

fn first_cluster_id(graph: &GraphViewModel) -> Option<String> {
    graph.nodes.iter().find_map(|node| {
        (node.parent.as_deref() == Some(graph.root_id.as_str())).then(|| node.id.clone())
    })
}

fn fallback_leaf_label(title: &str) -> String {
    if title.trim().is_empty() {
        "未命名条目".to_string()
    } else {
        title.trim().to_string()
    }
}

fn fallback_cluster_label(title: &str) -> String {
    if title.trim().is_empty() {
        "未命名分组".to_string()
    } else {
        title.trim().to_string()
    }
}

fn insert_graph_node(
    graph: &mut GraphViewModel,
    parent_id: &str,
    shape: NodeDraftShape,
    title: &str,
    body: &str,
    source: &str,
    tags: &[String],
) -> Option<String> {
    graph_node(graph, parent_id)?;

    let node_id = next_graph_node_id(graph, parent_id);
    let label = match shape {
        NodeDraftShape::Cluster => fallback_cluster_label(title),
        NodeDraftShape::Leaf => fallback_leaf_label(title),
    };
    let detail = if body.trim().is_empty() {
        match shape {
            NodeDraftShape::Cluster => "新建分组节点，后续可继续补充说明和子节点。".to_string(),
            NodeDraftShape::Leaf => "新建条目节点，后续可继续补充正文、来源和标签。".to_string(),
        }
    } else {
        body.trim().to_string()
    };
    let meta = compose_node_meta(source, tags);

    graph.nodes.push(GraphNodeItem {
        id: node_id.clone(),
        label,
        detail,
        meta,
        parent: Some(parent_id.to_string()),
        kind: match shape {
            NodeDraftShape::Cluster => GraphNodeKind::Cluster,
            NodeDraftShape::Leaf => GraphNodeKind::Leaf,
        },
    });
    graph.edges.push(GraphEdgeItem {
        source: parent_id.to_string(),
        target: node_id.clone(),
    });

    Some(node_id)
}

fn next_graph_node_id(graph: &GraphViewModel, parent_id: &str) -> String {
    let mut index = graph.nodes.len() + 1;
    loop {
        let candidate = format!("{parent_id}-{index}");
        if !graph.nodes.iter().any(|node| node.id == candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn update_graph_node(
    graph: &mut GraphViewModel,
    node_id: &str,
    title: &str,
    body: &str,
    source: &str,
    tags: &[String],
) {
    if let Some(node) = graph.nodes.iter_mut().find(|node| node.id == node_id) {
        if !title.trim().is_empty() {
            node.label = title.trim().to_string();
        }
        if !body.trim().is_empty() {
            node.detail = body.trim().to_string();
        }
        let meta = compose_node_meta(source, tags);
        if !meta.is_empty() {
            node.meta = meta;
        }
    }
}

fn compose_node_meta(source: &str, tags: &[String]) -> String {
    let source = source.trim();
    let tags = tags
        .iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    match (source.is_empty(), tags.is_empty()) {
        (true, true) => "本地暂存".to_string(),
        (false, true) => source.to_string(),
        (true, false) => tags.join(" / "),
        (false, false) => format!("{source} · {}", tags.join(" / ")),
    }
}

fn delete_graph_subtree(graph: &mut GraphViewModel, node_id: &str) {
    let children = child_map(&graph.nodes);
    let mut doomed = HashSet::from([node_id.to_string()]);
    include_descendants(node_id, &children, &mut doomed);
    graph
        .nodes
        .retain(|node| !doomed.contains(node.id.as_str()));
    graph.edges.retain(|edge| {
        !doomed.contains(edge.source.as_str()) && !doomed.contains(edge.target.as_str())
    });
}

fn note_docs() -> Vec<&'static KnowledgeDoc> {
    let rust_docs = KNOWLEDGE_DOCS
        .iter()
        .filter(|doc| doc.source_slug == "rust")
        .collect::<Vec<_>>();
    if rust_docs.is_empty() {
        KNOWLEDGE_DOCS
            .iter()
            .filter(|doc| doc.source_slug == "notes")
            .collect()
    } else {
        rust_docs
    }
}

fn note_clusters() -> Vec<NoteCluster> {
    let mut foundations = Vec::new();
    let mut core = Vec::new();
    let mut practice = Vec::new();
    let mut advanced = Vec::new();

    for doc in note_docs() {
        match note_cluster_for(doc) {
            "基础入门" => foundations.push(doc),
            "语言核心" => core.push(doc),
            "工程实践" => practice.push(doc),
            _ => advanced.push(doc),
        }
    }

    vec![
        NoteCluster {
            label: "基础入门",
            docs: foundations,
        },
        NoteCluster {
            label: "语言核心",
            docs: core,
        },
        NoteCluster {
            label: "工程实践",
            docs: practice,
        },
        NoteCluster {
            label: "高级专题",
            docs: advanced,
        },
    ]
    .into_iter()
    .filter(|cluster| !cluster.docs.is_empty())
    .collect()
}

fn note_cluster_for(doc: &KnowledgeDoc) -> &'static str {
    match note_chapter_no(doc) {
        1..=4 => "基础入门",
        5..=10 => "语言核心",
        11..=17 => "工程实践",
        _ => "高级专题",
    }
}

fn note_doc_chapter_label(doc: &KnowledgeDoc) -> String {
    let number = note_chapter_no(doc);
    if number == 0 {
        "CH".to_string()
    } else {
        format!("CH {:02}", number)
    }
}

fn note_chapter_no(doc: &KnowledgeDoc) -> usize {
    doc.filename
        .split('-')
        .next()
        .and_then(|raw| raw.parse::<usize>().ok())
        .unwrap_or_default()
}

fn truncate_copy(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx == limit {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.0} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{bytes} B")
    }
}
