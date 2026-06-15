use dioxus::prelude::*;

use crate::backend::model::{MasterDetailConfig, MetaFieldView, TableColumn};
use crate::ui::page::helpers::{
    parse_query, render_enum_select, render_enum_select_edit, render_rel_select,
    render_rel_select_edit, resolve_cell,
};
use crate::backend::record::{RecordStore, RecordWithId};

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
        section { class: "lowcode-page",
            div { style: "display:grid; grid-template-columns:240px 1fr; height:100vh; overflow:hidden;",
                // Left sidebar — tree
                aside { class: "lowcode-tree-panel",
                    div { style: "font-size:13px; font-weight:600; padding:8px 10px; border-bottom:1px solid var(--border-color, #e8e8e8);",
                        "记录树 ({all_records.len()})"
                    }
                    div { style: "overflow:auto; flex:1; padding:4px 0;",
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
                section { style: "display:flex; flex-direction:column; overflow:hidden;",
                    header { class: "lowcode-page__header",
                        h1 { "{title}" }
                        p {
                            if selected_id.is_empty() { "全部记录 · 共 {all_records.len()} 条" }
                            else { "选中节点 + 直接子项 · {display_records.len()} 条" }
                        }
                        a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }

                        if !selected_id.is_empty() && !parent_field.is_empty() {
                            details { class: "lowcode-accordion",
                                summary { class: "lowcode-accordion__summary", "＋ 添加子节点" }
                                div { class: "lowcode-accordion__body",
                                    form { method: "get", action: "/",
                                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                        input { r#type: "hidden", name: "action", value: "new-record" }
                                        input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                        input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                                        input { r#type: "hidden", name: "rec_{parent_field}", value: "{selected_id}" }
                                        for f in fields.iter() {
                                            if f.name != parent_field {
                                                div { class: "settings-form-row",
                                                    label { "{f.label}" }
                                                    input { class: "settings-input", name: "rec_{f.name}", placeholder: "输入{f.label}" }
                                                }
                                            }
                                        }
                                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "创建子节点" }
                                    }
                                }
                            }
                        }

                        details { class: "lowcode-accordion",
                            summary { class: "lowcode-accordion__summary", "＋ 新建记录" }
                            div { class: "lowcode-accordion__body",
                                form { method: "get", action: "/",
                                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                    input { r#type: "hidden", name: "action", value: "new-record" }
                                    input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                    input { r#type: "hidden", name: "screen", value: "{screen_id}" }
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

                        if !selected_id.is_empty() {
                            a {
                                href: "{action_base}",
                                class: "toolbar-button",
                                style: "font-size:11px;",
                                "显示全部"
                            }
                        }
                    }

                    div { style: "flex:1; overflow:auto;",
                        table { class: "az-table az-table--bordered az-table--dense",
                            thead {
                                tr {
                                    for col in &config.detail_columns {
                                        th { class: "az-table__header-cell", "{col.label}" }
                                    }
                                    th { class: "az-table__header-cell", style: "width:110px; text-align:center;", "操作" }
                                }
                            }
                            tbody { class: "az-table__body",
                                if display_records.is_empty() {
                                    tr { td { class: "az-table__cell az-table__cell--empty", colspan: "{config.detail_columns.len() + 1}", "暂无记录" } }
                                } else {
                                    for rec in &display_records {
                                        {
                                            let is_sel = rec.id == selected_id;
                                            rsx! {
                                                tr { style: if is_sel { "background:var(--highlight-bg, #e6f4ff);" } else { "" },
                                                    for col in &config.detail_columns {
                                                        td { class: "az-table__cell", style: if is_sel { "font-weight:600;" } else { "" },
                                                            "{resolve_cell(fields, &col.field_name, rec.fields.get(&col.field_name).cloned().unwrap_or_default())}"
                                                        }
                                                    }
                                                    td { class: "az-table__cell", style: "text-align:center; white-space:nowrap;",
                                                        details { class: "lowcode-accordion", style: "margin:0; display:inline-block;",
                                                            summary { class: "lowcode-accordion__summary", style: "font-size:11px; padding:2px 6px;", "编辑" }
                                                            div { class: "lowcode-accordion__body",
                                                                form { method: "get", action: "/",
                                                                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                                    input { r#type: "hidden", name: "action", value: "edit-record" }
                                                                    input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                                                    input { r#type: "hidden", name: "rec_id", value: "{rec.id}" }
                                                                    input { r#type: "hidden", name: "screen", value: "{screen_id}" }
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
                                                                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size:11px;", "保存" }
                                                                }
                                                            }
                                                        }
                                                        a {
                                                            href: "{action_base}&sel={selected_id}&action=delete-record&rec_model={model_id}&rec_id={rec.id}",
                                                            class: "toolbar-button toolbar-button--danger",
                                                            style: "font-size:11px; padding:2px 7px; margin-left:4px;",
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
