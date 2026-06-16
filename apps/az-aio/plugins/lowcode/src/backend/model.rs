use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_lowcode_";

// ── MetaModel ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct MetaModel {
    #[key]
    pub id: String,
    #[index]
    pub name: String,
    pub label: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetaModelSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub field_count: i64,
}

// ── MetaField ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Json,
    Relation,
    Enum,
}

impl FieldType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::String => "字符串",
            Self::Integer => "整数",
            Self::Float => "浮点数",
            Self::Boolean => "布尔",
            Self::DateTime => "日期时间",
            Self::Json => "JSON",
            Self::Relation => "关联",
            Self::Enum => "枚举",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToMany,
    SelfRecursive,
}

impl RelationType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OneToOne => "一对一",
            Self::OneToMany => "一对多",
            Self::ManyToMany => "多对多",
            Self::SelfRecursive => "自递归(树)",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct MetaField {
    #[key]
    pub id: String,
    #[index]
    pub model_id: String,
    pub name: String,
    pub label: String,
    pub field_type: String, // FieldType as string for toasty compat
    pub relation_type: Option<String>,
    pub relation_model_id: Option<String>,
    pub is_required: bool,
    pub is_unique: bool,
    pub order: i32,
    pub default_value: Option<String>,
    /// Comma-separated enum options, e.g. "draft,pending,active"
    pub enum_options: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetaFieldView {
    pub id: String,
    pub model_id: String,
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub relation_type: Option<String>,
    pub relation_model_id: Option<String>,
    pub relation_model_name: Option<String>,
    pub is_required: bool,
    pub is_unique: bool,
    pub order: i32,
    pub default_value: Option<String>,
    pub enum_options: Option<String>,
}

impl From<MetaField> for MetaFieldView {
    fn from(f: MetaField) -> Self {
        Self {
            id: f.id,
            model_id: f.model_id,
            name: f.name,
            label: f.label,
            field_type: f.field_type,
            relation_type: f.relation_type,
            relation_model_id: f.relation_model_id,
            relation_model_name: None,
            is_required: f.is_required,
            is_unique: f.is_unique,
            order: f.order,
            default_value: f.default_value,
            enum_options: f.enum_options,
        }
    }
}

// ── AppScreen ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenLayout {
    Table,
    MasterDetail,
    TreeTable,
    Accordion,
    Form,
}

impl ScreenLayout {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Table => "表格 · CRUD",
            Self::MasterDetail => "左树右表",
            Self::TreeTable => "树形表",
            Self::Accordion => "手风琴",
            Self::Form => "表单",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct AppScreen {
    #[key]
    pub id: String,
    #[index]
    pub name: String,
    pub label: String,
    pub layout: String,
    pub model_id: String,
    pub config_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppScreenSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub layout: String,
    pub model_id: String,
    pub model_name: String,
    pub created_at: String,
}

// ── AppScreen config schema ────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableConfig {
    pub columns: Vec<TableColumn>,
    #[serde(default)]
    pub searchable_fields: Vec<String>,
    #[serde(default)]
    pub page_size: usize,
    #[serde(default = "default_true")]
    pub frozen_header: bool,
    #[serde(default = "default_frozen_columns")]
    pub frozen_columns: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableColumn {
    pub field_name: String,
    pub label: String,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub width: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_frozen_columns() -> usize {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterDetailConfig {
    pub tree_field_id: String,
    pub detail_columns: Vec<TableColumn>,
    #[serde(default)]
    pub detail_searchable: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccordionConfig {
    pub groups: Vec<AccordionGroup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccordionGroup {
    pub label: String,
    pub fields: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormConfig {
    pub fields: Vec<FormField>,
    #[serde(default)]
    pub submit_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormField {
    pub field_name: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub options: Vec<String>,
}

// ── LowcodeRecord (persistent record storage) ─────────────────────

/// Persisted record row for lowcode-generated pages.
/// Fields are stored as JSON string for schema flexibility.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct LowcodeRecord {
    #[key]
    pub id: String,
    #[index]
    pub model_id: String,
    /// JSON-serialized field map: `{"name": "张三", "email": "..."}`
    pub fields_json: String,
    pub created_at: String,
    pub updated_at: String,
}
