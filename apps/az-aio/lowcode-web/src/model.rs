use serde::{Deserialize, Serialize};

// ── API response types (matching backend) ──────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetaModelSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub field_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetaModel {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppScreenSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub layout: String,
    pub model_id: String,
    pub model_name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppScreen {
    pub id: String,
    pub name: String,
    pub label: String,
    pub layout: String,
    pub model_id: String,
    pub config_json: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── Create/Update inputs ──────────────────────────────────────────

#[derive(Serialize)]
pub struct CreateModelInput {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Serialize)]
pub struct CreateFieldInput {
    pub name: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub relation_model_id: Option<String>,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Serialize)]
pub struct UpdateFieldInput {
    pub name: Option<String>,
    pub label: Option<String>,
    pub field_type: Option<String>,
    #[serde(default)]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub relation_model_id: Option<String>,
    pub is_required: Option<bool>,
    pub is_unique: Option<bool>,
    pub order: Option<i32>,
    pub default_value: Option<String>,
}

#[derive(Serialize)]
pub struct CreateScreenInput {
    pub name: String,
    pub label: String,
    pub layout: String,
    pub model_id: String,
    #[serde(default)]
    pub config_json: String,
}
