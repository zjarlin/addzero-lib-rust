use std::collections::HashMap;

use az_dioxus_components::{
    accordion::Accordion,
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{FormRow, Input},
    table::{Table, TableBody, TableCell, TableHead, TableHeaderCell, TableRow},
    workbench::{PageHeader, TableViewport, WorkbenchPage},
};
use dioxus::prelude::*;

use crate::backend::model::MetaFieldView;
use crate::backend::record::{RecordStore, RecordWithId};
use crate::ui::page::helpers::{
    render_enum_select, render_enum_select_edit, render_rel_select, render_rel_select_edit,
    resolve_cell, LowcodeActionForm,
};

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
        WorkbenchPage {
            PageHeader {
                title: title.to_string(),
                subtitle: format!("{} 条记录", records.len()),
                ToolbarButtonLink { href: format!("/?route={lowcode_route}&mode=screens"), "← 返回" }

                Accordion { title: "＋ 新建记录",
                        LowcodeActionForm {
                            action_name: "new-record",
                            hidden_fields: vec![("rec_model".to_string(), model_id.to_string())],
                            for f in fields.iter() {
                                FormRow { label: f.label.clone(),
                                    if f.field_type == "Enum" {
                                        {render_enum_select(f)}
                                    } else if f.field_type == "Relation" {
                                        {render_rel_select(f, &rec_store)}
                                    } else {
                                        Input { name: format!("rec_{}", f.name), placeholder: format!("输入{}", f.label) }
                                    }
                                }
                            }
                            ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "创建" }
                        }
                }
            }
            TableViewport {
                Table { bordered: true, dense: true,
                    TableHead {
                        TableRow {
                            TableHeaderCell { style: "width: 40px;", "" }
                            for f in fields.iter().filter(|f| f.field_type != "Relation") {
                                TableHeaderCell { "{f.label}" }
                            }
                            TableHeaderCell { style: "width: 110px; text-align: center;", "操作" }
                        }
                    }
                    TableBody {
                        if records.is_empty() {
                            TableRow {
                                TableCell { class: "table-view__cell--empty", colspan: col_len, "暂无记录" }
                            }
                        } else if has_tree {
                            {tree_rows(&roots, &children, label_field, fields, model_id, &rec_store, lowcode_route, 0)}
                        } else {
                            for rec in &records {
                                TableRow {
                                    TableCell { "📄" }
                                    for cn in &col_names {
                                        TableCell {
                                            "{resolve_cell(fields, cn, rec.fields.get(*cn).cloned().unwrap_or_default())}"
                                        }
                                    }
                                    TableCell { style: "text-align: center; white-space: nowrap;",
                                        Accordion { title: "编辑", class: "accordion--inline", summary_class: "accordion__summary--compact",
                                                LowcodeActionForm {
                                                    action_name: "edit-record",
                                                    hidden_fields: vec![
                                                        ("rec_model".to_string(), model_id.to_string()),
                                                        ("rec_id".to_string(), rec.id.clone()),
                                                    ],
                                                    for fv in fields.iter() {
                                                        FormRow { label: fv.label.clone(),
                                                            if fv.field_type == "Enum" {
                                                                {render_enum_select_edit(fv, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                            } else if fv.field_type == "Relation" {
                                                                {render_rel_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                            } else {
                                                                Input {
                                                                    name: format!("rec_{}", fv.name),
                                                                    value: rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default(),
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "保存" }
                                                }
                                        }
                                        ToolbarButtonLink {
                                            href: format!("/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={}", rec.id),
                                            tone: ToolbarButtonTone::Danger,
                                            class: "toolbar-button--table-gap",
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
            TableRow {
                TableCell { style: "padding-left: {pad_left}px;",
                    span { "{connector} {icon}" }
                    strong { " {label}" }
                }
                for cn in &col_names {
                    TableCell {
                        "{resolve_cell(fields, cn, node.fields.get(*cn).cloned().unwrap_or_default())}"
                    }
                }
                TableCell { style: "text-align: center; white-space: nowrap;",
                    Accordion { title: "编辑", class: "accordion--inline", summary_class: "accordion__summary--compact",
                            LowcodeActionForm {
                                action_name: "edit-record",
                                hidden_fields: vec![
                                    ("rec_model".to_string(), model_id.to_string()),
                                    ("rec_id".to_string(), node.id.clone()),
                                ],
                                for fv in fields.iter() {
                                    FormRow { label: fv.label.clone(),
                                        if fv.field_type == "Enum" {
                                            {render_enum_select_edit(fv, node.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                        } else if fv.field_type == "Relation" {
                                            {render_rel_select_edit(fv, rec_store, node.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                        } else {
                                            Input {
                                                name: format!("rec_{}", fv.name),
                                                value: node.fields.get(fv.name.as_str()).cloned().unwrap_or_default(),
                                            }
                                        }
                                    }
                                }
                                ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "保存" }
                            }
                    }
                    ToolbarButtonLink {
                        href: format!("/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={}", node.id),
                        tone: ToolbarButtonTone::Danger,
                        class: "toolbar-button--table-gap",
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
