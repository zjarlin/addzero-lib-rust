use dioxus::prelude::*;

use crate::model::{MetaFieldView, TableColumn, TableConfig};
use crate::page::helpers::resolve_cell;
use crate::record::RecordStore;

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
    let records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";
    let col_names: Vec<&str> = config.columns.iter().map(|c| c.field_name.as_str()).collect();
    let col_len = col_names.len() + 1;

    // Parse screen_id from query for action redirects
    let screen_id = parse_q(query, "screen").unwrap_or_default();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "增删改查表格 — {records.len()} 条记录" }
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
                                    "暂无记录 — 使用上方表单创建"
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
                                        details { class: "lowcode-accordion",
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
                                                    for cn2 in &col_names {
                                                        div { class: "settings-form-row",
                                                            label { "{cn2}" }
                                                            input { class: "settings-input", name: "rec_{cn2}", value: "{rec.fields.get(*cn2).cloned().unwrap_or_default()}", style: "font-size: 12px;" }
                                                        }
                                                    }
                                                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                                                }
                                            }
                                        }
                                        a {
                                            href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}&screen={screen_id}",
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

/// Render a <select> dropdown for a Relation field, populated from the related model's records.
fn relation_select(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
    if let Some(ref rel_model_id) = field.relation_model_id {
        let options = rec_store.list(rel_model_id);
        rsx! {
            select { class: "settings-input", name: "rec_{field.name}",
                option { value: "", "— 选择 —" }
                for opt in &options {
                    option {
                        value: "{opt.id}",
                        "{opt.fields.get(\"name\").or(opt.fields.get(\"label\")).cloned().unwrap_or_else(|| opt.id.clone())}"
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

fn parse_q<'a>(query: &'a str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts.next().map(|v| urlencoding::decode(v).unwrap_or_else(|_| v.into()).into());
        }
    }
    None
}
