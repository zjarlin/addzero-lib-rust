use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Capture,
    Note,
    Skill,
    Software,
    Package,
}

impl AssetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Note => "note",
            Self::Skill => "skill",
            Self::Software => "software",
            Self::Package => "package",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "capture" => Self::Capture,
            "skill" => Self::Skill,
            "software" => Self::Software,
            "package" => Self::Package,
            _ => Self::Note,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetEdgeUpsert {
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relation: String,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetGraph {
    pub assets: Vec<Asset>,
    pub edges: Vec<AssetEdge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    OpenAi,
    Anthropic,
    Gemini,
}

impl AiProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "anthropic" => Self::Anthropic,
            "gemini" => Self::Gemini,
            _ => Self::OpenAi,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiModelProvider {
    pub provider: AiProviderKind,
    pub default_model: String,
    pub enabled: bool,
    pub key_id: String,
    pub api_key_configured: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiModelProviderUpsert {
    pub provider: AiProviderKind,
    pub default_model: String,
    pub enabled: bool,
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetProviderSecret {
    pub provider: AiProviderKind,
    pub default_model: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiPromptButtonUpsert {
    pub id: Option<Uuid>,
    pub label: String,
    pub target_kind: AssetKind,
    pub prompt_template: String,
    pub provider: AiProviderKind,
    pub model: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SuggestedEdge {
    pub target_title: String,
    pub relation: String,
    pub confidence: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptRunOutput {
    pub title: String,
    pub tags: Vec<String>,
    pub body: String,
    pub suggested_edges: Vec<SuggestedEdge>,
}
