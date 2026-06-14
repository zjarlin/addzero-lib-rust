use std::collections::HashMap;

use dioxus::prelude::*;

use crate::model::MetaFieldView;
use crate::page::helpers::resolve_cell;
use crate::record::{RecordStore, RecordWithId};

pub fn render_master_detail(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
    _config_json: &str,
    query: &str,
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
    let has_tree = parent_field.is_some();
    let parent_key = parent_field
        .map(|f| f.name.as_str())
        .unwrap_or("parent_id");

    // Build children map
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

    // Read expand param to filter detail table
    let expand = parse_q(query, "expand");
    let screen_id = parse_q(query, "screen").unwrap_or_default();

    fn collect_descendant_ids(
        parent_id: &str,
        children: &HashMap<String, Vec<&RecordWithId>>,
        out: &mut Vec<String>,
    ) {
        if let Some(kids) = children.get(parent_id) {
            for kid in kids {
                out.push(kid.id.clone());
                collect_descendant_ids(&kid.id, children, out);
            }
        }
    }

    let filtered_records: Vec<&RecordWithId> = match expand {
        Some(ref expand_root) => {
            let mut ids = vec![expand_root.clone()];
            collect_descendant_ids(expand_root, &children, &mut ids);
            records.iter().filter(|r| ids.contains(&r.id)).collect()
        }
        None => records.iter().collect(),
    };

    let detail_cols: Vec<&MetaFieldView> =
        fields.iter().filter(|f| f.field_type != "Relation").collect();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "左树右表 — 点击树节点过滤右侧数据" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
            }
            div { class: "lowcode-workbench",
                aside { class: "lowcode-tree",
                    div { class: "lowcode-tree__header", h2 { "导航" } }
                    div { class: "lowcode-tree__list",
                        if has_tree {
                            // "全部" link to reset filter
                            a {
                                href: "/?route={lowcode_route}&screen={screen_id}",
                                class: if expand.is_none() { "nav-button nav-button--active" } else { "nav-button" },
                                span { "📂 全部 ({records.len()})" }
                            }
                            {render_tree_nodes(&roots, &children, label_field, 0, lowcode_route, &screen_id, expand.as_deref())}
                        } else {
                            for rec in &records {
                                div { class: "nav-button",
                                    "{rec.fields.get(label_field).cloned().unwrap_or_default()}"
                                }
                            }
                        }
                    }
                }
                section { class: "lowcode-detail",
                    div { class: "lowcode-detail__header",
                        h2 {
                            if let Some(ref root_id) = expand {
                                if let Some(r) = records.iter().find(|r| r.id == *root_id) {
                                    "{r.fields.get(label_field).cloned().unwrap_or_default()} — 子项"
                                }
                            } else {
                                "全部记录"
                            }
                        }
                        span { class: "lowcode-detail__subtitle", "{filtered_records.len()} 条" }
                    }
                    div { class: "lowcode-table-scroll",
                        table { class: "az-table az-table--bordered az-table--dense",
                            thead {
                                tr {
                                    for f in &detail_cols {
                                        th { class: "az-table__header-cell", "{f.label}" }
                                    }
                                    th { class: "az-table__header-cell", "操作" }
                                }
                            }
                            tbody { class: "az-table__body",
                                if filtered_records.is_empty() {
                                    tr {
                                        td { class: "az-table__cell az-table__cell--empty", colspan: "{detail_cols.len() + 1}", "暂无记录" }
                                    }
                                } else {
                                    for rec in &filtered_records {
                                        tr {
                                            for f in &detail_cols {
                                                td { class: "az-table__cell",
                                                    "{resolve_cell(fields, f.name.as_str(), rec.fields.get(f.name.as_str()).cloned().unwrap_or_default())}"
                                                }
                                            }
                                            td { class: "az-table__cell",
                                                a {
                                                    href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}&screen={screen_id}&expand={expand.as_deref().unwrap_or(\"\")}",
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
    }
}

fn render_tree_nodes(
    nodes: &[&RecordWithId],
    children: &HashMap<String, Vec<&RecordWithId>>,
    label_field: &str,
    depth: usize,
    lowcode_route: &str,
    screen_id: &str,
    active_expand: Option<&str>,
) -> Element {
    let pad = depth * 16 + 12;
    rsx! {
        for node in nodes {
            div {
                a {
                    class: if active_expand == Some(node.id.as_str()) { "nav-button nav-button--active" } else { "nav-button" },
                    style: "padding-left: {pad}px;",
                    href: "/?route={lowcode_route}&screen={screen_id}&expand={node.id}",
                    span { "📁 " }
                    span { "{node.fields.get(label_field).cloned().unwrap_or_default()}" }
                }
                if let Some(kids) = children.get(&node.id) {
                    {render_tree_nodes(kids, children, label_field, depth + 1, lowcode_route, screen_id, active_expand)}
                }
            }
        }
    }
}

fn parse_q<'a>(query: &'a str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts.next().map(|v| urlencoding::decode(v).unwrap_or_else(|_| v.into()).into());
        }
    }
    None
}
