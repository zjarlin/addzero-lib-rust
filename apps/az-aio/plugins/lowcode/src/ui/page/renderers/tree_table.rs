use std::collections::HashMap;

use dioxus::prelude::*;

use crate::backend::model::MetaFieldView;
use crate::ui::page::helpers::{
    render_enum_select, render_enum_select_edit, render_rel_select, render_rel_select_edit,
    resolve_cell,
};
use crate::backend::record::{RecordStore, RecordWithId};

pub fn render_tree_table(title: &str, model_id: &str, fields: &[MetaFieldView]) -> Element {
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

    let parent_key = parent_field.map(|f| f.name.as_str()).unwrap_or("parent_id");
    let has_tree = parent_field.is_some();

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

    let col_names: Vec<&str> = fields
        .iter()
        .filter(|f| f.field_type != "Relation")
        .map(|f| f.name.as_str())
        .collect();
    let col_len = col_names.len() + 1;

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header", style: "padding: 10px 16px 8px;",
                h1 { style: "font-size: 17px; font-weight: 640; margin: 0;", "{title}" }
                p { "{records.len()} 条记录" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }

                details { class: "lowcode-accordion",
                    summary { class: "lowcode-accordion__summary", "＋ 新建记录" }
                    div { class: "lowcode-accordion__body",
                        form { method: "get", action: "/",
                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                            input { r#type: "hidden", name: "action", value: "new-record" }
                            input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                            for f in fields.iter() {
                                div { class: "settings-form-row",
                                    label { "{f.label}" }
                                    if f.field_type == "Enum" {
                                        {render_enum_select(f)}
                                    } else if f.field_type == "Relation" {
                                        {render_rel_select(f, &rec_store)}
                                    } else {
                                        input { class: "settings-input", name: "rec_{f.name}", placeholder: "输入{f.label}" }
                                    }
                                }
                            }
                            button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "创建" }
                        }
                    }
                }
            }
            div { class: "lowcode-table-scroll",
                table { class: "az-table az-table--bordered az-table--dense",
                    thead {
                        tr {
                            th { class: "az-table__header-cell", style: "width: 40px;", "" }
                            for f in fields.iter().filter(|f| f.field_type != "Relation") {
                                th { class: "az-table__header-cell", "{f.label}" }
                            }
                            th { class: "az-table__header-cell", style: "width: 110px; text-align: center;", "操作" }
                        }
                    }
                    tbody { class: "az-table__body",
                        if records.is_empty() {
                            tr {
                                td { class: "az-table__cell az-table__cell--empty", colspan: "{col_len}", "暂无记录" }
                            }
                        } else if has_tree {
                            {tree_rows(&roots, &children, label_field, fields, model_id, &rec_store, lowcode_route, 0)}
                        } else {
                            for rec in &records {
                                tr {
                                    td { class: "az-table__cell", "📄" }
                                    for cn in &col_names {
                                        td { class: "az-table__cell",
                                            "{resolve_cell(fields, cn, rec.fields.get(*cn).cloned().unwrap_or_default())}"
                                        }
                                    }
                                    td { class: "az-table__cell", style: "text-align: center; white-space: nowrap;",
                                        details { class: "lowcode-accordion", style: "margin: 0; display: inline-block;",
                                            summary { class: "lowcode-accordion__summary", style: "font-size: 11px; padding: 2px 6px;", "编辑" }
                                            div { class: "lowcode-accordion__body",
                                                form { method: "get", action: "/",
                                                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                    input { r#type: "hidden", name: "action", value: "edit-record" }
                                                    input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                                    input { r#type: "hidden", name: "rec_id", value: "{rec.id}" }
                                                    for fv in fields.iter() {
                                                        div { class: "settings-form-row",
                                                            label { "{fv.label}" }
                                                            if fv.field_type == "Enum" {
                                                                {render_enum_select_edit(fv, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                            } else if fv.field_type == "Relation" {
                                                                {render_rel_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                            } else {
                                                                input { class: "settings-input", name: "rec_{fv.name}", value: "{rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default()}" }
                                                            }
                                                        }
                                                    }
                                                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                                                }
                                            }
                                        }
                                        a {
                                            href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}",
                                            class: "toolbar-button toolbar-button--danger",
                                            style: "font-size: 11px; padding: 2px 7px; margin-left: 4px;",
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

fn tree_rows(
    nodes: &[&RecordWithId],
    children: &HashMap<String, Vec<&RecordWithId>>,
    label_field: &str,
    fields: &[MetaFieldView],
    model_id: &str,
    rec_store: &RecordStore,
    lowcode_route: &str,
    depth: usize,
) -> Element {
    let col_names: Vec<&str> = fields
        .iter()
        .filter(|f| f.field_type != "Relation")
        .map(|f| f.name.as_str())
        .collect();
    let total = nodes.len();

    let row_els: Vec<Element> = nodes.iter().enumerate().map(|(i, node)| {
        let is_last = i == total - 1;
        let connector = if depth > 0 && is_last { "└─" } else if depth > 0 { "├─" } else { "" };
        let pad_left = depth * 24;

        let has_children = children.contains_key(&node.id);
        let label = node.fields.get(label_field).cloned().unwrap_or_else(|| node.id.clone());
        let icon = if has_children { "📁" } else { "📄" };

        rsx! {
            tr {
                td { class: "az-table__cell", style: "padding-left: {pad_left}px;",
                    span { "{connector} {icon}" }
                    strong { " {label}" }
                }
                for cn in &col_names {
                    td { class: "az-table__cell",
                        "{resolve_cell(fields, cn, node.fields.get(*cn).cloned().unwrap_or_default())}"
                    }
                }
                td { class: "az-table__cell", style: "text-align: center; white-space: nowrap;",
                    details { class: "lowcode-accordion", style: "margin: 0; display: inline-block;",
                        summary { class: "lowcode-accordion__summary", style: "font-size: 11px; padding: 2px 6px;", "编辑" }
                        div { class: "lowcode-accordion__body",
                            form { method: "get", action: "/",
                                input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                input { r#type: "hidden", name: "action", value: "edit-record" }
                                input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                input { r#type: "hidden", name: "rec_id", value: "{node.id}" }
                                for fv in fields.iter() {
                                    div { class: "settings-form-row",
                                        label { "{fv.label}" }
                                        if fv.field_type == "Enum" {
                                            {render_enum_select_edit(fv, node.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                        } else if fv.field_type == "Relation" {
                                            {render_rel_select_edit(fv, rec_store, node.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                        } else {
                                            input { class: "settings-input", name: "rec_{fv.name}", value: "{node.fields.get(fv.name.as_str()).cloned().unwrap_or_default()}" }
                                        }
                                    }
                                }
                                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                            }
                        }
                    }
                    a {
                        href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={node.id}",
                        class: "toolbar-button toolbar-button--danger",
                        style: "font-size: 11px; padding: 2px 7px; margin-left: 4px;",
                        "删除"
                    }
                }
            }
            // Recursively render children
            if let Some(kids) = children.get(&node.id) {
                {tree_rows(kids, children, label_field, fields, model_id, rec_store, lowcode_route, depth + 1)}
            }
        }
    }).collect();

    rsx! { {row_els.into_iter().map(|r| r)} }
}
