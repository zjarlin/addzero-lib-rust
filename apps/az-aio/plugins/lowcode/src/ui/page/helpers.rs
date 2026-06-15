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
        select { class: "settings-input", name: "rec_{field.name}",
            option { value: "", "— 选择 —" }
            for opt in &opts { option { value: "{opt}", "{opt}" } }
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
        select { class: "settings-input", name: "rec_{field.name}",
            option { value: "", "— 选择 —" }
            for opt in &opts { option { value: "{opt}", selected: *opt == current, "{opt}" } }
        }
    }
}

/// Render a Relation-type field as a dropdown for create forms.
pub fn render_rel_select(field: &MetaFieldView, rec_store: &RecordStore) -> Element {
    if let Some(ref rel_id) = field.relation_model_id {
        let opts = rec_store.list(rel_id);
        rsx! {
            select { class: "settings-input", name: "rec_{field.name}",
                option { value: "", "— 选择 —" }
                for o in &opts { option { value: "{o.id}", "{opt_display_label(o)}" } }
            }
        }
    } else {
        rsx! { input { class: "settings-input", name: "rec_{field.name}", placeholder: "关联ID" } }
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
            select { class: "settings-input", name: "rec_{field.name}",
                option { value: "", "— 选择 —" }
                for o in &opts { option { value: "{o.id}", selected: o.id == current, "{opt_display_label(o)}" } }
            }
        }
    } else {
        rsx! { input { class: "settings-input", name: "rec_{field.name}", value: "{current}" } }
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
