use az_derive_aliases::{
    apply, serde_code_default_enum, serde_eq, serde_partial_eq, serde_partial_eq_default,
};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[apply(serde_code_default_enum)]
pub enum AssetKind {
    Capture,
    #[default]
    Note,
    Skill,
    Software,
    Package,
}

impl AssetKind {
    pub fn from_db_value(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

#[apply(serde_eq)]
pub struct Asset {
    pub id: Uuid,
    pub kind: AssetKind,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_partial_eq)]
pub struct AssetEdge {
    pub id: Uuid,
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relation: String,
    pub confidence: f64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct AssetUpsert {
    pub id: Option<Uuid>,
    pub kind: AssetKind,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub metadata: serde_json::Value,
}

impl AssetUpsert {
    pub fn compute_hash(&self) -> String {
        let mut tags = self.tags.clone();
        tags.sort();
        let mut hasher = Sha256::new();
        hasher.update(self.kind.as_str().as_bytes());
        hasher.update(b"\0");
        hasher.update(self.title.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.body.as_bytes());
        hasher.update(b"\0");
        hasher.update(tags.join(",").as_bytes());
        hasher.update(b"\0");
        hasher.update(self.metadata.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[apply(serde_partial_eq)]
pub struct AssetEdgeUpsert {
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relation: String,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

#[apply(serde_partial_eq_default)]
pub struct AssetGraph {
    pub assets: Vec<Asset>,
    pub edges: Vec<AssetEdge>,
}

#[apply(serde_code_default_enum)]
pub enum AiProviderKind {
    #[default]
    #[serde(rename = "openai")]
    #[strum(serialize = "openai")]
    OpenAi,
    Anthropic,
    Gemini,
}

impl AiProviderKind {
    pub fn from_db_value(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

#[apply(serde_eq)]
pub struct AiModelProvider {
    pub provider: AiProviderKind,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub key_id: String,
    pub api_key_configured: bool,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct AiModelProviderUpsert {
    pub provider: AiProviderKind,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub api_key: Option<String>,
}

#[apply(serde_eq)]
pub struct AssetProviderSecret {
    pub provider: AiProviderKind,
    pub base_url: Option<String>,
    pub default_model: String,
    pub api_key: String,
}

#[apply(serde_eq)]
pub struct AiPromptButton {
    pub id: Uuid,
    pub label: String,
    pub target_kind: AssetKind,
    pub prompt_template: String,
    pub provider: AiProviderKind,
    pub model: String,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct AiPromptButtonUpsert {
    pub id: Option<Uuid>,
    pub label: String,
    pub target_kind: AssetKind,
    pub prompt_template: String,
    pub provider: AiProviderKind,
    pub model: String,
    pub enabled: bool,
}

#[apply(serde_eq)]
pub struct SuggestedEdge {
    pub target_title: String,
    pub relation: String,
    pub confidence: u8,
}

#[apply(serde_eq)]
pub struct PromptRunOutput {
    pub title: String,
    pub tags: Vec<String>,
    pub body: String,
    pub suggested_edges: Vec<SuggestedEdge>,
}
