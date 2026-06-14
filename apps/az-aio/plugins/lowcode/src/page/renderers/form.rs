use dioxus::prelude::*;

use crate::model::{FormConfig, FormField, MetaFieldView};
use crate::page::helpers::ft_html;
use crate::record::RecordStore;

pub fn render_form(
    title: &str,
    model_id: &str,
    meta_fields: &[MetaFieldView],
    config_json: &str,
) -> Element {
    let config: FormConfig = serde_json::from_str(config_json).unwrap_or_else(|_| {
        FormConfig {
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
        }
    });
    let lowcode_route = "/lowcode";
    let rec_store = RecordStore::global();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "表单布局" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }
            }
            form {
                method: "get",
                action: "/",
                div { style: "padding: 16px 20px; max-width: 640px;",
                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                    input { r#type: "hidden", name: "action", value: "new-record" }
                    input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                    for field in &config.fields {
                        // Find the matching MetaFieldView for Relation info
                        let meta = meta_fields.iter().find(|mf| mf.name == field.field_name);
                        div { class: "settings-form-row",
                            label {
                                "{field.label}"
                                if field.required {
                                    span { style: "color: var(--warning); margin-left: 4px;", "*" }
                                }
                            }
                            if let Some(mf) = meta {
                                if mf.field_type == "Relation" {
                                    {render_relation_select(mf, &rec_store)}
                                } else if field.options.is_empty() {
                                    input {
                                        class: "settings-input",
                                        name: "rec_{field.field_name}",
                                        r#type: ft_html(&field.field_type),
                                        placeholder: "{field.placeholder}",
                                    }
                                } else {
                                    select { class: "settings-input", name: "rec_{field.field_name}",
                                        for opt in &field.options { option { value: "{opt}", "{opt}" } }
                                    }
                                }
                            } else if field.options.is_empty() {
                                input {
                                    class: "settings-input",
                                    name: "rec_{field.field_name}",
                                    r#type: ft_html(&field.field_type),
                                    placeholder: "{field.placeholder}",
                                }
                            } else {
                                select { class: "settings-input", name: "rec_{field.field_name}",
                                    for opt in &field.options { option { value: "{opt}", "{opt}" } }
                                }
                            }
                        }
                    }
                    div { style: "margin-top: 18px;",
                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "{config.submit_label}" }
                    }
                }
            }
        }
    }
}

fn render_relation_select(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
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
