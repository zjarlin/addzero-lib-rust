use crate::model::MetaFieldView;
use crate::record::RecordStore;
use crate::store::LowcodeStore;

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
