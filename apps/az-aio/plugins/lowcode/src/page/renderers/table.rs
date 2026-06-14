use dioxus::prelude::*;

use crate::model::{MetaFieldView, TableColumn, TableConfig};
use crate::page::helpers::resolve_cell;
use crate::record::{RecordStore, RecordWithId};

pub fn render_table_screen(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
    config_json: &str,
    query: &str,
) -> Element {
    let config: TableConfig = serde_json::from_str(config_json).unwrap_or_else(|_| {
        TableConfig {
            columns: fields
                .iter()
                .filter(|f| f.field_type != "Relation")
                .map(|f| TableColumn {
                    field_name: f.name.clone(),
                    label: f.label.clone(),
                    sortable: false,
                    width: None,
                })
                .collect(),
            searchable_fields: vec![],
            page_size: 20,
        }
    });
    let rec_store = RecordStore::global();
    let all_records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";
    let col_names: Vec<&str> = config.columns.iter().map(|c| c.field_name.as_str()).collect();
    let col_len = col_names.len() + 1;

    let search = parse_q(query, "search").unwrap_or_default();
    let screen_id = parse_q(query, "screen").unwrap_or_default();

    let records: Vec<_> = if search.is_empty() {
        all_records.iter().collect()
    } else {
        let q = search.to_lowercase();
        all_records
            .iter()
            .filter(|r| r.fields.values().any(|v| v.to_lowercase().contains(&q)))
            .collect()
    };

    // Pre-build relation field info for edit forms

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header", style: "padding: 10px 16px 8px;",
                h1 { style: "font-size: 17px; font-weight: 640; margin: 0;", "{title}" }
                p { "{records.len()} 条记录" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
                details { class: "lowcode-accordion",
                    summary { class: "lowcode-accordion__summary", "＋ 新建记录" }
                    div { class: "lowcode-accordion__body",
                        form {
                            method: "get",
                            action: "/",
                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                            input { r#type: "hidden", name: "action", value: "new-record" }
                            input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                            input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                            for f in fields.iter() {
                                div { class: "settings-form-row",
                                    label { "{f.label}" }
                                    if f.field_type == "Relation" {
                                        {relation_select(f, &rec_store)}
                                    } else {
                                        input {
                                            class: "settings-input",
                                            name: "rec_{f.name}",
                                            placeholder: "输入{f.label}",
                                        }
                                    }
                                }
                            }
                            button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "创建" }
                        }
                    }
                }
                form {
                    method: "get",
                    action: "/",
                    div { style: "display: flex; gap: 8px; align-items: center; padding: 8px 0;",
                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                        input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                        input {
                            class: "settings-input",
                            name: "search",
                            placeholder: "搜索...",
                            value: "{search}",
                            style: "max-width: 280px;",
                        }
                        button { class: "toolbar-button", r#type: "submit", "搜索" }
                        if !search.is_empty() {
                            a {
                                href: "/?route={lowcode_route}&screen={screen_id}",
                                class: "toolbar-button",
                                style: "font-size: 11px;",
                                "清除"
                            }
                        }
                    }
                }
            }
            div { class: "lowcode-table-scroll",
                table { class: "az-table az-table--bordered az-table--dense",
                    thead {
                        tr {
                            for col in &config.columns {
                                th { class: "az-table__header-cell", "{col.label}" }
                            }
                            th { class: "az-table__header-cell", "操作" }
                        }
                    }
                    tbody { class: "az-table__body",
                        if records.is_empty() {
                            tr {
                                td { class: "az-table__cell az-table__cell--empty", colspan: "{col_len}",
                                    if search.is_empty() {
                                        "暂无记录 — 使用上方表单创建"
                                    } else {
                                        "无匹配记录"
                                    }
                                }
                            }
                        } else {
                            for rec in &records {
                                tr {
                                    for cn in &col_names {
                                        td { class: "az-table__cell",
                                            "{resolve_cell(fields, *cn, rec.fields.get(*cn).cloned().unwrap_or_default())}"
                                        }
                                    }
                                    td { class: "az-table__cell",
                                        div { style: "display: flex; gap: 4px; align-items: center;",
                                            details { class: "lowcode-accordion", style: "margin: 0;",
                                                summary { class: "lowcode-accordion__summary", style: "font-size: 11px; padding: 2px 6px;", "编辑" }
                                                div { class: "lowcode-accordion__body",
                                                    form {
                                                        method: "get",
                                                        action: "/",
                                                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                        input { r#type: "hidden", name: "action", value: "edit-record" }
                                                        input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                                        input { r#type: "hidden", name: "rec_id", value: "{rec.id}" }
                                                        input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                                                        for fv in fields.iter() {
                                                            div { class: "settings-form-row",
                                                                label { "{fv.label}" }
                                                                if fv.field_type == "Relation" {
                                                                    {relation_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                                } else {
                                                                    input { class: "settings-input", name: "rec_{fv.name}", value: "{rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default()}", style: "font-size: 12px;" }
                                                                }
                                                            }
                                                        }
                                                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                                                    }
                                                }
                                            }
                                            a {
                                                href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}&screen={screen_id}",
                                                class: "toolbar-button toolbar-button--danger",
                                                style: "font-size: 11px; padding: 2px 7px;",
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

fn relation_select(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
    if let Some(ref rel_model_id) = field.relation_model_id {
        let options = rec_store.list(rel_model_id);
        rsx! {
            select { class: "settings-input", name: "rec_{field.name}",
                option { value: "", "— 选择 —" }
                for opt in &options {
                    option {
                        value: "{opt.id}",
                        "{opt_label(opt)}"
                    }
                }
            }
        }
    } else {
        rsx! {
            input {
                class: "settings-input",
                name: "rec_{field.name}",
                placeholder: "关联ID",
            }
        }
    }
}

fn relation_select_edit(field: &MetaFieldView, rec_store: &RecordStore, current_value: String) -> Element {
    if let Some(ref rel_model_id) = field.relation_model_id {
        let options = rec_store.list(rel_model_id);
        rsx! {
            select { class: "settings-input", name: "rec_{field.name}",
                option { value: "", "— 选择 —" }
                for opt in &options {
                    option {
                        value: "{opt.id}",
                        selected: opt.id == current_value,
                        "{opt_label(opt)}"
                    }
                }
            }
        }
    } else {
        rsx! {
            input {
                class: "settings-input",
                name: "rec_{field.name}",
                value: "{current_value}",
                placeholder: "关联ID",
            }
        }
    }
}

fn opt_label(opt: &RecordWithId) -> String {
    opt.fields
        .get("name")
        .or(opt.fields.get("label"))
        .cloned()
        .unwrap_or_else(|| opt.id.clone())
}

fn parse_q<'a>(query: &'a str, key: &str) -> Option<String> {
    let qs = query.split('?').nth(1).unwrap_or(query);
    for pair in qs.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts
                .next()
                .map(|v| urlencoding::decode(v).unwrap_or_else(|_| v.into()).into());
        }
    }
    None
}
