use std::collections::HashMap;

use dioxus::prelude::*;

use crate::model::MetaFieldView;
use crate::record::{RecordStore, RecordWithId};

pub fn render_tree_table(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
) -> Element {
    let rec_store = RecordStore::global();
    let records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";

    let parent_field = fields
        .iter()
        .find(|f| f.relation_type.as_deref() == Some("SelfRecursive"));
    let label_field = fields
        .iter()
        .find(|f| f.field_type == "String" && f.name != "parent_id")
        .map(|f| f.name.as_str())
        .unwrap_or("name");

    let parent_key = parent_field
        .map(|f| f.name.as_str())
        .unwrap_or("parent_id");
    let has_tree = parent_field.is_some();

    // Build tree from records
    let mut children: HashMap<String, Vec<&RecordWithId>> = HashMap::new();
    let mut roots: Vec<&RecordWithId> = Vec::new();
    for rec in &records {
        let pid = rec.fields.get(parent_key).cloned().unwrap_or_default();
        if pid.is_empty() {
            roots.push(rec);
        } else {
            children.entry(pid).or_default().push(rec);
        }
    }

    // Non-relation display columns
    let display_cols: Vec<&MetaFieldView> =
        fields.iter().filter(|f| f.field_type != "Relation").collect();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "树形表格 — {records.len()} 条记录" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
            }
            div { class: "lowcode-table-scroll",
                table { class: "az-table az-table--bordered az-table--dense",
                    thead {
                        tr {
                            th { class: "az-table__header-cell", style: "width: 40px;", "" }
                            for f in &display_cols {
                                th { class: "az-table__header-cell", "{f.label}" }
                            }
                            th { class: "az-table__header-cell", "操作" }
                        }
                    }
                    tbody { class: "az-table__body",
                        if records.is_empty() {
                            tr {
                                td { class: "az-table__cell az-table__cell--empty", colspan: "{display_cols.len() + 2}", "暂无记录" }
                            }
                        } else if has_tree {
                            {render_tree_rows(&roots, &children, label_field, &display_cols, model_id, 0)}
                        } else {
                            for rec in &records {
                                tr {
                                    td { class: "az-table__cell", "📄" }
                                    for f in &display_cols {
                                        td { class: "az-table__cell",
                                            "{rec.fields.get(f.name.as_str()).cloned().unwrap_or_default()}"
                                        }
                                    }
                                    td { class: "az-table__cell",
                                        a {
                                            href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}",
                                            class: "toolbar-button toolbar-button--danger",
                                            "删除"
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
}

fn render_tree_rows(
    nodes: &[&RecordWithId],
    children: &HashMap<String, Vec<&RecordWithId>>,
    label_field: &str,
    display_cols: &[&MetaFieldView],
    model_id: &str,
    depth: usize,
) -> Element {
    let lowcode_route = "/lowcode";
    let pipe = "│  ".repeat(depth.saturating_sub(1));
    let _prefix = if depth > 0 {
        format!("{pipe}├─")
    } else {
        String::new()
    };
    let _icon = if children.contains_key(&nodes.first().map(|n| n.id.clone()).unwrap_or_default())
        || nodes.iter().any(|n| children.contains_key(&n.id))
    {
        "📁"
    } else {
        "📄"
    };

    // We need to iterate and produce Element for each node, handling tree nesting.
    // But rsx! can't do internal recursive loops well. Use a flat approach:
    let rows: Vec<Element> = tree_rows_flat(
        nodes,
        children,
        label_field,
        display_cols,
        model_id,
        depth,
        lowcode_route,
    );
    rsx! { {rows.into_iter().map(|r| r)} }
}

fn tree_rows_flat(
    nodes: &[&RecordWithId],
    children: &HashMap<String, Vec<&RecordWithId>>,
    label_field: &str,
    display_cols: &[&MetaFieldView],
    model_id: &str,
    depth: usize,
    lowcode_route: &str,
) -> Vec<Element> {
    let mut out: Vec<Element> = Vec::new();
    let total = nodes.len();

    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if depth > 0 && is_last { "└─" } else if depth > 0 { "├─" } else { "" };
        let pad_left = depth * 24;

        let has_children = children.contains_key(&node.id);

        let label = node
            .fields
            .get(label_field)
            .cloned()
            .unwrap_or_else(|| node.id.clone());
        let icon = if has_children { "📁" } else { "📄" };

        let row_el = rsx! {
            tr {
                td { class: "az-table__cell", style: "padding-left: {pad_left}px;",
                    span { "{connector} {icon}" }
                    strong { " {label}" }
                }
                for f in display_cols {
                    td { class: "az-table__cell",
                        "{node.fields.get(f.name.as_str()).cloned().unwrap_or_default()}"
                    }
                }
                td { class: "az-table__cell",
                    a {
                        href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={node.id}",
                        class: "toolbar-button toolbar-button--danger",
                        "删除"
                    }
                }
            }
        };
        out.push(row_el);

        if let Some(kids) = children.get(&node.id) {
            let kid_rows = tree_rows_flat(
                kids,
                children,
                label_field,
                display_cols,
                model_id,
                depth + 1,
                lowcode_route,
            );
            out.extend(kid_rows);
        }
    }
    out
}
