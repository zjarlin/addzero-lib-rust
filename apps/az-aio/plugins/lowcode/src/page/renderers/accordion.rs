use dioxus::prelude::*;

use crate::model::MetaFieldView;
use crate::page::helpers::resolve_cell;
use crate::record::RecordStore;

pub fn render_accordion(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
) -> Element {
    let rec_store = RecordStore::global();
    let records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header", style: "padding: 10px 16px 8px;",
                h1 { style: "font-size: 17px; font-weight: 640; margin: 0;", "{title}" }
                p { "{records.len()} 条记录" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
                // Create form
                details { class: "lowcode-accordion",
                    summary { class: "lowcode-accordion__summary", "＋ 新建记录" }
                    div { class: "lowcode-accordion__body",
                        form {
                            method: "get",
                            action: "/",
                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                            input { r#type: "hidden", name: "action", value: "new-record" }
                            input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                            for f in fields.iter() {
                                div { class: "settings-form-row",
                                    label { "{f.label}" }
                                    if f.field_type == "Relation" {
                                        {relation_select_acc(f, &rec_store)}
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
            div { style: "padding: 16px 20px;",
                if records.is_empty() {
                    p { class: "az-platform-muted", "暂无记录" }
                } else {
                    for rec in &records {
                        details { class: "lowcode-accordion",
                            summary { class: "lowcode-accordion__summary",
                                div { style: "display: flex; align-items: center; justify-content: space-between;",
                                    span { "{label_from_record(rec, fields)}" }
                                    a {
                                        href: "/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={rec.id}",
                                        class: "toolbar-button toolbar-button--danger",
                                        style: "font-size: 11px; padding: 2px 8px;",
                                        "删除"
                                    }
                                }
                            }
                            div { class: "lowcode-accordion__body",
                                for f in fields.iter() {
                                    div { class: "settings-form-row",
                                        label {
                                            "{f.label}"
                                            if f.field_type == "Relation" {
                                                span { class: "az-badge az-badge--accent", style: "margin-left: 4px; font-size: 10px;", "关联" }
                                            }
                                        }
                                        div { class: "lowcode-field-value",
                                            if f.field_type == "Relation" {
                                                "{resolve_cell(fields, f.name.as_str(), rec.fields.get(f.name.as_str()).cloned().unwrap_or_default())}"
                                            } else {
                                                "{rec.fields.get(f.name.as_str()).cloned().unwrap_or_default()}"
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

fn relation_select_acc(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
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

/// Derive a display label from the record's first String field, or fall back to the record ID.
fn label_from_record(
    rec: &crate::record::RecordWithId,
    fields: &[MetaFieldView],
) -> String {
    if let Some(f) = fields
        .iter()
        .find(|f| f.field_type == "String" && f.name != "parent_id")
    {
        return rec
            .fields
            .get(f.name.as_str())
            .cloned()
            .unwrap_or_else(|| rec.id.clone());
    }
    rec.id.clone()
}
