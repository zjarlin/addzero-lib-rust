use az_dioxus_components::{
    az_accordion::AzAccordion,
    az_badge::{AzBadge, AzBadgeTone},
    az_button::{AzButton, AzButtonLink, AzButtonTone},
    az_form::{AzFormRow, AzInput},
    az_workbench::{AzPageHeader, AzWorkbenchPage},
};
use dioxus::prelude::*;

use crate::backend::model::MetaFieldView;
use crate::backend::record::RecordStore;
use crate::ui::page::helpers::{
    render_enum_select, render_enum_select_edit, render_rel_select, render_rel_select_edit,
    resolve_cell, LowcodeActionForm,
};

pub fn render_accordion(title: &str, model_id: &str, fields: &[MetaFieldView]) -> Element {
    let rec_store = RecordStore::global();
    let records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";

    rsx! {
        AzWorkbenchPage {
            AzPageHeader {
                title: title.to_string(),
                subtitle: format!("{} 条记录", records.len()),
                AzButtonLink { href: format!("/?route={lowcode_route}&mode=screens"), "← 返回" }
                AzAccordion { title: "＋ 新建记录",
                        LowcodeActionForm {
                            action_name: "new-record",
                            hidden_fields: vec![("rec_model".to_string(), model_id.to_string())],
                            for f in fields.iter() {
                                AzFormRow { label: f.label.clone(),
                                    if f.field_type == "Enum" {
                                        {render_enum_select(f)}
                                    } else if f.field_type == "Relation" {
                                        {render_rel_select(f, &rec_store)}
                                    } else {
                                        AzInput { name: format!("rec_{}", f.name), placeholder: format!("输入{}", f.label) }
                                    }
                                }
                            }
                            AzButton { tone: AzButtonTone::Primary, button_type: "submit", "创建" }
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
                                    AzButtonLink {
                                        href: format!("/?route={lowcode_route}&action=delete-record&rec_model={model_id}&rec_id={}", rec.id),
                                        tone: AzButtonTone::Danger,
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
                                                AzBadge { tone: AzBadgeTone::Accent, class: "az-badge--inline", "关联" }
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
                                AzAccordion { title: "编辑", summary_class: "az-accordion__summary--compact",
                                        LowcodeActionForm {
                                            action_name: "edit-record",
                                            hidden_fields: vec![
                                                ("rec_model".to_string(), model_id.to_string()),
                                                ("rec_id".to_string(), rec.id.clone()),
                                            ],
                                            for fv in fields.iter() {
                                                AzFormRow { label: fv.label.clone(),
                                                    if fv.field_type == "Enum" {
                                                        {render_enum_select_edit(fv, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                    } else if fv.field_type == "Relation" {
                                                        {render_rel_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                    } else {
                                                        AzInput {
                                                            name: format!("rec_{}", fv.name),
                                                            value: rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default(),
                                                        }
                                                    }
                                                }
                                            }
                                            AzButton { tone: AzButtonTone::Primary, button_type: "submit", "保存" }
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
fn label_from_record(rec: &crate::backend::record::RecordWithId, fields: &[MetaFieldView]) -> String {
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
