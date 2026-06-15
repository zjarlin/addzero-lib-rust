use dioxus::prelude::*;

use crate::backend::model::{FormConfig, FormField, MetaFieldView};
use crate::ui::page::helpers::{render_enum_select, render_rel_select};
use crate::backend::record::RecordStore;

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

            let label_el = rsx! {
                label {
                    "{field.label}"
                    if field.required {
                        span { style: "color: var(--warning); margin-left: 4px;", "*" }
                    }
                }
            };

            let input_el = if is_enum {
                if let Some(mf) = meta {
                    render_enum_select(mf)
                } else {
                    rsx! { input { class: "settings-input", name: "rec_{field.field_name}", placeholder: "{field.placeholder}" } }
                }
            } else if is_rel {
                if let Some(mf) = meta {
                    render_rel_select(mf, &rec_store)
                } else {
                    rsx! { input { class: "settings-input", name: "rec_{field.field_name}", placeholder: "关联ID" } }
                }
            } else if field.options.is_empty() {
                rsx! {
                    input {
                        class: "settings-input",
                        name: "rec_{field.field_name}",
                        r#type: ft_html_simple(&field.field_type),
                        placeholder: "{field.placeholder}",
                    }
                }
            } else {
                rsx! {
                    select { class: "settings-input", name: "rec_{field.field_name}",
                        for opt in &field.options { option { value: "{opt}", "{opt}" } }
                    }
                }
            };

            rsx! {
                div { class: "settings-form-row",
                    {label_el}
                    {input_el}
                }
            }
        })
        .collect();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header", style: "padding: 10px 16px 8px;",
                h1 { style: "font-size: 17px; font-weight: 640; margin: 0;", "{title}" }
                p { "表单" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
            }
            form {
                method: "get",
                action: "/",
                div { style: "padding: 16px 20px; max-width: 640px;",
                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                    input { r#type: "hidden", name: "action", value: "new-record" }
                    input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                    {field_els.into_iter().map(|el| el)}
                    div { style: "margin-top: 18px;",
                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "{config.submit_label}" }
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
