use az_dioxus_components::{
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{FormRow, Input, Select, SelectOption},
    workbench::{PageHeader, WorkbenchPage},
};
use dioxus::prelude::*;

use crate::backend::model::{FormConfig, FormField, MetaFieldView};
use crate::backend::record::RecordStore;
use crate::ui::page::helpers::{render_enum_select, render_rel_select, LowcodeActionForm};

pub fn render_form(
    title: &str,
    model_id: &str,
    meta_fields: &[MetaFieldView],
    config_json: &str,
) -> Element {
    let config: FormConfig = serde_json::from_str(config_json).unwrap_or_else(|_| FormConfig {
        fields: meta_fields
            .iter()
            .map(|f| FormField {
                field_name: f.name.clone(),
                label: f.label.clone(),
                field_type: f.field_type.clone(),
                required: f.is_required,
                placeholder: format!("输入{}", f.label),
                options: vec![],
            })
            .collect(),
        submit_label: "保存".into(),
    });
    let lowcode_route = "/lowcode";
    let rec_store = RecordStore::global();

    let field_els: Vec<Element> = config
        .fields
        .iter()
        .map(|field| {
            let meta = meta_fields.iter().find(|mf| mf.name == field.field_name);
            let is_rel = meta.map(|m| m.field_type.as_str() == "Relation").unwrap_or(false);
            let is_enum = meta.map(|m| m.field_type.as_str() == "Enum").unwrap_or(false);

            let input_el = if is_enum {
                if let Some(mf) = meta {
                    render_enum_select(mf)
                } else {
                    rsx! { Input { name: format!("rec_{}", field.field_name), placeholder: field.placeholder.clone() } }
                }
            } else if is_rel {
                if let Some(mf) = meta {
                    render_rel_select(mf, &rec_store)
                } else {
                    rsx! { Input { name: format!("rec_{}", field.field_name), placeholder: "关联ID" } }
                }
            } else if field.options.is_empty() {
                rsx! {
                    Input {
                        name: format!("rec_{}", field.field_name),
                        input_type: ft_html_simple(&field.field_type),
                        placeholder: field.placeholder.clone(),
                    }
                }
            } else {
                rsx! {
                    Select {
                        name: format!("rec_{}", field.field_name),
                        options: field.options.iter().map(|option| SelectOption::new(option.clone(), option.clone())).collect::<Vec<_>>(),
                    }
                }
            };

            rsx! {
                FormRow { label: field.label.clone(), required: field.required,
                    {input_el}
                }
            }
        })
        .collect();

    rsx! {
        WorkbenchPage {
            PageHeader { title: title.to_string(), subtitle: "表单",
                ToolbarButtonLink { href: format!("/?route={lowcode_route}&mode=screens"), "← 返回" }
            }
            LowcodeActionForm {
                action_name: "new-record",
                hidden_fields: vec![("rec_model".to_string(), model_id.to_string())],
                div { style: "padding: 16px 20px; max-width: 640px;",
                    {field_els.into_iter().map(|el| el)}
                    div { style: "margin-top: 18px;",
                        ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "{config.submit_label}" }
                    }
                }
            }
        }
    }
}

fn ft_html_simple(ft: &str) -> &str {
    match ft {
        "integer" | "Integer" => "number",
        "float" | "Float" => "number",
        "boolean" | "Boolean" => "checkbox",
        "datetime" | "DateTime" => "datetime-local",
        _ => "text",
    }
}
