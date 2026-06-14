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
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "手风琴布局 — {records.len()} 条记录" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
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
