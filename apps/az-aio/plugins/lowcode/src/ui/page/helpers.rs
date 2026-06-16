use crate::backend::model::MetaFieldView;
use crate::backend::record::RecordStore;
use crate::backend::store::LowcodeStore;

pub fn get_store() -> LowcodeStore {
    LowcodeStore::global()
}

/// If a field is a Relation type, resolve the display label from the related model's records.
pub fn resolve_cell(fields: &[MetaFieldView], col_name: &str, raw_value: String) -> String {
    if raw_value.is_empty() {
        return "—".into();
    }
    if let Some(f) = fields.iter().find(|f| f.name == col_name) {
        if f.field_type == "Relation" {
            if let Some(ref rel_model_id) = f.relation_model_id {
                let recs = RecordStore::global().list(rel_model_id);
                if let Some(r) = recs.iter().find(|r| r.id == raw_value) {
                    return r
                        .fields
                        .get("name")
                        .or(r.fields.get("label"))
                        .cloned()
                        .unwrap_or(raw_value);
                }
            }
        }
    }
    raw_value
}

pub fn ft_label(ft: &str) -> &str {
    match ft {
        "String" => "字符串",
        "Integer" => "整数",
        "Float" => "浮点数",
        "Boolean" => "布尔",
        "DateTime" => "日期时间",
        "Json" => "JSON",
        "Relation" => "关联",
        _ => ft,
    }
}

pub fn rel_label(rt: Option<&str>) -> &str {
    match rt {
        Some("OneToOne") => "一对一",
        Some("OneToMany") => "一对多",
        Some("ManyToMany") => "多对多",
        Some("SelfRecursive") => "自递归",
        _ => "关联",
    }
}

pub fn layout_label(layout: &str) -> &str {
    match layout {
        "Table" => "增删改查表格",
        "MasterDetail" => "左树右表",
        "TreeTable" => "树形表格",
        "Accordion" => "手风琴",
        "Form" => "表单",
        _ => layout,
    }
}

pub fn ft_html(ft: &str) -> &str {
    match ft {
        "integer" | "Integer" => "number",
        "float" | "Float" => "number",
        "boolean" | "Boolean" => "checkbox",
        "datetime" | "DateTime" => "datetime-local",
        _ => "text",
    }
}

use crate::backend::record::RecordWithId;
use az_dioxus_components::az_form::{AzActionForm, AzHiddenInput, AzInput, AzSelect, AzSelectOption};
use dioxus::prelude::*;

/// Extract a query parameter value from a route string (e.g. "?key=val&...").
pub fn parse_query(route: &str, key: &str) -> Option<String> {
    let qs = route.split('?').nth(1).unwrap_or(route);
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

/// Render a lowcode SSR action form with stable route/action hidden fields.
#[allow(non_snake_case)]
#[component]
pub fn LowcodeActionForm(
    children: Element,
    #[props(into)] action_name: String,
    #[props(default = "/lowcode".to_string(), into)] route: String,
    #[props(default)] hidden_fields: Vec<(String, String)>,
    #[props(default, into)] id: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
) -> Element {
    rsx! {
        AzActionForm { id: id, class: class, style: style,
            AzHiddenInput { name: "route", value: route }
            AzHiddenInput { name: "action", value: action_name }
            for (name, value) in hidden_fields {
                AzHiddenInput { name: name, value: value }
            }
            {children}
        }
    }
}

/// Render an Enum-type field as a dropdown for create forms.
pub fn render_enum_select(field: &MetaFieldView) -> Element {
    let opts: Vec<&str> = field
        .enum_options
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    rsx! {
        AzSelect {
            name: format!("rec_{}", field.name),
            options: select_options(opts, None),
        }
    }
}

/// Render an Enum-type field as a dropdown for edit forms, with current value pre-selected.
pub fn render_enum_select_edit(field: &MetaFieldView, current: String) -> Element {
    let opts: Vec<&str> = field
        .enum_options
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    rsx! {
        AzSelect {
            name: format!("rec_{}", field.name),
            options: select_options(opts, Some(current)),
        }
    }
}

/// Render a Relation-type field as a dropdown for create forms.
pub fn render_rel_select(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
    if let Some(ref rel_id) = field.relation_model_id {
        let opts = rec_store.list(rel_id);
        rsx! {
            AzSelect {
                name: format!("rec_{}", field.name),
                options: relation_options(&opts, None),
            }
        }
    } else {
        rsx! { AzInput { name: format!("rec_{}", field.name), placeholder: "关联ID" } }
    }
}

/// Render a Relation-type field as a dropdown for edit forms, with current value pre-selected.
pub fn render_rel_select_edit(
    field: &MetaFieldView,
    rec_store: &RecordStore,
    current: String,
) -> Element {
    if let Some(ref rel_id) = field.relation_model_id {
        let opts = rec_store.list(rel_id);
        rsx! {
            AzSelect {
                name: format!("rec_{}", field.name),
                options: relation_options(&opts, Some(current)),
            }
        }
    } else {
        rsx! { AzInput { name: format!("rec_{}", field.name), value: current } }
    }
}

/// Derive a human-readable label from a record (first available String field, or its ID).
pub fn opt_display_label(opt: &RecordWithId) -> String {
    opt.fields
        .get("name")
        .or(opt.fields.get("label"))
        .cloned()
        .unwrap_or_else(|| opt.id.clone())
}

fn select_options(options: Vec<&str>, current: Option<String>) -> Vec<AzSelectOption> {
    std::iter::once(AzSelectOption::new("", "— 选择 —"))
        .chain(options.into_iter().map(|option| {
            AzSelectOption::new(option, option)
                .selected(current.as_deref().is_some_and(|value| value == option))
        }))
        .collect()
}

fn relation_options(options: &[RecordWithId], current: Option<String>) -> Vec<AzSelectOption> {
    std::iter::once(AzSelectOption::new("", "— 选择 —"))
        .chain(options.iter().map(|option| {
            AzSelectOption::new(option.id.clone(), opt_display_label(option))
                .selected(current.as_deref() == Some(option.id.as_str()))
        }))
        .collect()
}
