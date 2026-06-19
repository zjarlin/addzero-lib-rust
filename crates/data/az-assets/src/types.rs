use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 资产节点的业务类型。
///
/// 该枚举通过稳定 code 参与 API 传输和数据库存储，未知 code 默认回落到 [`AssetKind::Note`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AssetKind {
    Capture,
    #[default]
    Note,
    Skill,
    Software,
    Package,
}

impl AssetKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 资产图谱中的节点快照。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    /// 资产主键。
    pub id: Uuid,
    /// 资产类型。
    pub kind: AssetKind,
    /// 展示标题。
    pub title: String,
    /// 正文内容。
    pub body: String,
    /// 用于检索和分组的标签。
    pub tags: Vec<String>,
    /// 业务状态，例如 `active`。
    pub status: String,
    /// 扩展元数据。
    pub metadata: serde_json::Value,
    /// 基于类型、标题、正文、标签和元数据计算的内容哈希。
    pub content_hash: String,
    /// 创建时间。
    pub created_at: DateTime<Utc>,
    /// 最后更新时间。
    pub updated_at: DateTime<Utc>,
}

/// 资产节点之间的有向关系边。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetEdge {
    /// 边主键。
    pub id: Uuid,
    /// 源资产 id。
    pub source_asset_id: Uuid,
    /// 目标资产 id。
    pub target_asset_id: Uuid,
    /// 关系名称，例如 `relates_to` 或 `stored_in`。
    pub relation: String,
    /// 关系置信度。
    pub confidence: f64,
    /// 扩展元数据。
    pub metadata: serde_json::Value,
    /// 创建时间。
    pub created_at: DateTime<Utc>,
    /// 最后更新时间。
    pub updated_at: DateTime<Utc>,
}

/// 创建或更新资产节点的输入 DTO。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetUpsert {
    /// 有值时更新现有资产；为空时创建新资产。
    pub id: Option<Uuid>,
    /// 资产类型。
    pub kind: AssetKind,
    /// 展示标题。
    pub title: String,
    /// 正文内容。
    pub body: String,
    /// 标签列表。
    pub tags: Vec<String>,
    /// 业务状态。
    pub status: String,
    /// 扩展元数据。
    pub metadata: serde_json::Value,
}

impl AssetUpsert {
    /// 计算用于变更检测和去重的稳定内容哈希。
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

/// 创建或更新资产关系边的输入 DTO。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetEdgeUpsert {
    /// 源资产 id。
    pub source_asset_id: Uuid,
    /// 目标资产 id。
    pub target_asset_id: Uuid,
    /// 关系名称。
    pub relation: String,
    /// 关系置信度。
    pub confidence: f64,
    /// 扩展元数据。
    pub metadata: serde_json::Value,
}

/// 资产节点和关系边组成的完整图谱快照。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssetGraph {
    /// 图谱节点。
    pub assets: Vec<Asset>,
    /// 图谱边。
    pub edges: Vec<AssetEdge>,
}

/// AI 模型 provider 类型。
///
/// code 值用于 API、数据库和 agent 默认模型映射，新增 provider 时需要同步 agent 层。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AiProviderKind {
    #[default]
    #[serde(rename = "openai")]
    #[strum(serialize = "openai")]
    OpenAi,
    Anthropic,
    Gemini,
}

impl AiProviderKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 已保存的 AI 模型 provider 配置。
///
/// 该类型不暴露明文 API key，只通过 `api_key_configured` 表示密钥是否已配置。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AiModelProvider {
    /// provider 类型。
    pub provider: AiProviderKind,
    /// 自定义 provider base URL。
    pub base_url: Option<String>,
    /// 默认模型名。
    pub default_model: String,
    /// 是否启用该 provider。
    pub enabled: bool,
    /// 加密主密钥标识。
    pub key_id: String,
    /// 是否已保存 API key。
    pub api_key_configured: bool,
    /// 最后更新时间。
    pub updated_at: DateTime<Utc>,
}

/// 创建或更新 AI 模型 provider 的输入 DTO。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AiModelProviderUpsert {
    /// provider 类型。
    pub provider: AiProviderKind,
    /// 自定义 provider base URL。
    pub base_url: Option<String>,
    /// 默认模型名。
    pub default_model: String,
    /// 是否启用该 provider。
    pub enabled: bool,
    /// 新 API key；为空时保留已有密钥。
    pub api_key: Option<String>,
}

/// 解密后的 provider 运行凭据。
///
/// 该类型只应在服务端调用 provider 前短暂存在，不应返回给前端或持久化为明文。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AssetProviderSecret {
    /// provider 类型。
    pub provider: AiProviderKind,
    /// 自定义 provider base URL。
    pub base_url: Option<String>,
    /// 默认模型名。
    pub default_model: String,
    /// 解密后的 API key。
    pub api_key: String,
}

/// 可由前端触发的 AI prompt 按钮配置。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AiPromptButton {
    /// prompt 按钮 id。
    pub id: Uuid,
    /// 按钮展示文案。
    pub label: String,
    /// prompt 输出目标资产类型。
    pub target_kind: AssetKind,
    /// prompt 模板。
    pub prompt_template: String,
    /// 使用的 AI provider。
    pub provider: AiProviderKind,
    /// 使用的模型名。
    pub model: String,
    /// 是否启用。
    pub enabled: bool,
    /// 最后更新时间。
    pub updated_at: DateTime<Utc>,
}

/// 创建或更新 prompt 按钮的输入 DTO。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AiPromptButtonUpsert {
    /// 有值时更新现有按钮；为空时创建新按钮。
    pub id: Option<Uuid>,
    /// 按钮展示文案。
    pub label: String,
    /// prompt 输出目标资产类型。
    pub target_kind: AssetKind,
    /// prompt 模板。
    pub prompt_template: String,
    /// 使用的 AI provider。
    pub provider: AiProviderKind,
    /// 使用的模型名。
    pub model: String,
    /// 是否启用。
    pub enabled: bool,
}

/// AI 或规则推断出的候选关系边。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SuggestedEdge {
    /// 候选目标资产标题。
    pub target_title: String,
    /// 候选关系名称。
    pub relation: String,
    /// 置信度，范围约定为 `0..=100`。
    pub confidence: u8,
}

/// prompt 或本地规则运行后的结构化输出。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PromptRunOutput {
    /// 推断标题。
    pub title: String,
    /// 推断标签。
    pub tags: Vec<String>,
    /// 规范化后的正文。
    pub body: String,
    /// 候选图谱边。
    pub suggested_edges: Vec<SuggestedEdge>,
}
