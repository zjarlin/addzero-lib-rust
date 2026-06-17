use az_dioxus_components::{
    accordion::Accordion,
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{FormRow, Input},
    table::{Table, TableBody, TableCell, TableHead, TableHeaderCell, TableRow},
    workbench::{
        PageHeader, SplitWorkbench, TableViewport, WorkbenchDetail, WorkbenchPage,
        WorkbenchTree, WorkbenchTreeHeader, WorkbenchTreeList,
    },
};
use dioxus::prelude::*;

use crate::backend::model::{MasterDetailConfig, MetaFieldView, TableColumn};
use crate::backend::record::{RecordStore, RecordWithId};
use crate::ui::page::helpers::{
    parse_query, render_enum_select, render_enum_select_edit, render_rel_select,
    render_rel_select_edit, resolve_cell, LowcodeActionForm,
};

pub fn render_master_detail(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
    config_json: &str,
    query: &str,
) -> Element {
    let config: MasterDetailConfig =
        serde_json::from_str(config_json).unwrap_or_else(|_| MasterDetailConfig {
            tree_field_id: fields
                .iter()
                .find(|f| f.relation_type.as_deref() == Some("SelfRecursive"))
                .map(|f| f.id.clone())
                .unwrap_or_default(),
            detail_columns: fields
                .iter()
                .filter(|f| f.field_type != "Relation")
                .map(|f| TableColumn {
                    field_name: f.name.clone(),
                    label: f.label.clone(),
                    sortable: false,
                    width: None,
                })
                .collect(),
            detail_searchable: vec![],
        });

    let rec_store = RecordStore::global();
    let all_records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";
    let screen_id = parse_query(query, "screen").unwrap_or_default();
    let selected_id = parse_query(query, "sel").unwrap_or_default();
    let _search = parse_query(query, "search").unwrap_or_default();

    let label_field = fields
        .first()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "name".into());
    let parent_field = fields
        .iter()
        .find(|f| f.relation_type.as_deref() == Some("SelfRecursive"))
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "parent_id".into());

    // Build tree structure
    let has_tree = fields
        .iter()
        .any(|f| f.relation_type.as_deref() == Some("SelfRecursive"));

    // Filter detail records
    let display_records: Vec<&RecordWithId> = if selected_id.is_empty() {
        all_records.iter().collect()
    } else {
        all_records
            .iter()
            .filter(|r| {
                let rid = &r.id;
                let pid = r
                    .fields
                    .get(&parent_field)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                rid == &selected_id || pid == selected_id
            })
            .collect()
    };

    let action_base = format!("/?route={lowcode_route}&screen={screen_id}");

    rsx! {
        WorkbenchPage {
            SplitWorkbench { class: "split-workbench--master-detail",
                // Left sidebar — tree
                WorkbenchTree {
                    WorkbenchTreeHeader { title: format!("记录树 ({})", all_records.len()) }
                    WorkbenchTreeList {
                        if has_tree {
                            {render_tree_nodes(&all_records, &label_field, &parent_field, "", 0, &selected_id, &action_base)}
                        } else {
                            for rec in &all_records {
                                {
                                    let label = rec.fields.get(&label_field).cloned().unwrap_or_else(|| rec.id.clone());
                                    let is_sel = rec.id == selected_id;
                                    rsx! {
                                        a {
                                            href: "{action_base}&sel={rec.id}",
                                            class: if is_sel { "lowcode-tree-item lowcode-tree-item--active" } else { "lowcode-tree-item" },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right detail
                WorkbenchDetail {
                    PageHeader {
                        title: title.to_string(),
                        subtitle: if selected_id.is_empty() {
                            format!("全部记录 · 共 {} 条", all_records.len())
                        } else {
                            format!("选中节点 + 直接子项 · {} 条", display_records.len())
                        },
                        ToolbarButtonLink { href: format!("/?route={lowcode_route}&mode=screens"), "← 返回" }

                        if !selected_id.is_empty() && !parent_field.is_empty() {
                            Accordion { title: "＋ 添加子节点",
                                    LowcodeActionForm {
                                        action_name: "new-record",
                                        hidden_fields: vec![
                                            ("rec_model".to_string(), model_id.to_string()),
                                            ("screen".to_string(), screen_id.clone()),
                                            (format!("rec_{parent_field}"), selected_id.clone()),
                                        ],
                                        for f in fields.iter() {
                                            if f.name != parent_field {
                                                FormRow { label: f.label.clone(),
                                                    Input { name: format!("rec_{}", f.name), placeholder: format!("输入{}", f.label) }
                                                }
                                            }
                                        }
                                        ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "创建子节点" }
                                    }
                            }
                        }

                        Accordion { title: "＋ 新建记录",
                                LowcodeActionForm {
                                    action_name: "new-record",
                                    hidden_fields: vec![
                                        ("rec_model".to_string(), model_id.to_string()),
                                        ("screen".to_string(), screen_id.clone()),
                                    ],
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

                        if !selected_id.is_empty() {
                            ToolbarButtonLink {
                                href: action_base.clone(),
                                "显示全部"
                            }
                        }
                    }

                    TableViewport {
                        Table { bordered: true, dense: true,
                            TableHead {
                                TableRow {
                                    for col in &config.detail_columns {
                                        TableHeaderCell { "{col.label}" }
                                    }
                                    TableHeaderCell { style: "width:110px; text-align:center;", "操作" }
                                }
                            }
                            TableBody {
                                if display_records.is_empty() {
                                    TableRow {
                                        TableCell { class: "table-view__cell--empty", colspan: config.detail_columns.len() + 1, "暂无记录" }
                                    }
                                } else {
                                    for rec in &display_records {
                                        {
                                            let is_sel = rec.id == selected_id;
                                            rsx! {
                                                TableRow { style: if is_sel { "background:var(--highlight-bg, #e6f4ff);" } else { "" },
                                                    for col in &config.detail_columns {
                                                        TableCell { style: if is_sel { "font-weight:600;" } else { "" },
                                                            "{resolve_cell(fields, &col.field_name, rec.fields.get(&col.field_name).cloned().unwrap_or_default())}"
                                                        }
                                                    }
                                                    TableCell { style: "text-align:center; white-space:nowrap;",
                                                        Accordion { title: "编辑", class: "accordion--inline", summary_class: "accordion__summary--compact",
                                                                LowcodeActionForm {
                                                                    action_name: "edit-record",
                                                                    hidden_fields: vec![
                                                                        ("rec_model".to_string(), model_id.to_string()),
                                                                        ("rec_id".to_string(), rec.id.clone()),
                                                                        ("screen".to_string(), screen_id.clone()),
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
                                                            href: format!("{action_base}&sel={selected_id}&action=delete-record&rec_model={model_id}&rec_id={}", rec.id),
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
            }
        }
    }
}

fn render_tree_nodes(
    records: &[RecordWithId],
    label_field: &str,
    parent_field: &str,
    parent_id: &str,
    depth: usize,
    selected_id: &str,
    action_base: &str,
) -> Element {
    let children: Vec<&RecordWithId> = records
        .iter()
        .filter(|r| r.fields.get(parent_field).map(|s| s.as_str()).unwrap_or("") == parent_id)
        .collect();

    if children.is_empty() {
        return rsx! {};
    }

    rsx! {
        for rec in &children {
            {
                let label = rec.fields.get(label_field).cloned().unwrap_or_else(|| rec.id.clone());
                let is_sel = rec.id == selected_id;
                let indent = depth * 16;
                rsx! {
                    a {
                        href: "{action_base}&sel={rec.id}",
                        class: if is_sel { "lowcode-tree-item lowcode-tree-item--active" } else { "lowcode-tree-item" },
                        style: "padding-left:{indent + 10}px;",
                        span { style: "font-size:10px; color:var(--text-secondary, #999); margin-right:4px;",
                            if depth > 0 { "└" } else { "" }
                        }
                        "{label}"
                    }
                    {render_tree_nodes(records, label_field, parent_field, &rec.id, depth + 1, selected_id, action_base)}
                }
            }
        }
    }
}
