use std::{future::Future, pin::Pin, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiAssetKindDto {
    Capture,
    #[default]
    Note,
    Skill,
    Software,
    Package,
}

impl AiAssetKindDto {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Note => "note",
            Self::Skill => "skill",
            Self::Software => "software",
            Self::Package => "package",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Capture => "采集",
            Self::Note => "笔记",
            Self::Skill => "Skill",
            Self::Software => "软件",
            Self::Package => "安装包",
        }
    }

    pub fn from_query(value: &str) -> Option<Self> {
        Some(match value {
            "capture" => Self::Capture,
            "note" => Self::Note,
            "skill" => Self::Skill,
            "software" => Self::Software,
            "package" => Self::Package,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKindDto {
    #[default]
    OpenAi,
    Anthropic,
    Gemini,
}

impl AiProviderKindDto {
    pub const ALL: [Self; 3] = [Self::OpenAi, Self::Anthropic, Self::Gemini];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Gemini",
        }
    }

    pub fn from_query(value: &str) -> Option<Self> {
        Some(match value {
            "openai" => Self::OpenAi,
            "anthropic" => Self::Anthropic,
            "gemini" => Self::Gemini,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiAssetDto {
    pub id: String,
    pub kind: AiAssetKindDto,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub metadata: Value,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiAssetUpsertDto {
    pub id: Option<String>,
    pub kind: AiAssetKindDto,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SuggestedEdgeDto {
    pub target_title: String,
    pub relation: String,
    pub confidence: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiCaptureRequestDto {
    pub raw_content: String,
    pub target_kind: AiAssetKindDto,
    pub prompt_button_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiCaptureResponseDto {
    pub capture_asset: AiAssetDto,
    pub generated_asset: AiAssetDto,
    pub suggested_edges: Vec<SuggestedEdgeDto>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiModelProviderDto {
    pub provider: AiProviderKindDto,
    pub label: String,
    pub default_model: String,
    pub enabled: bool,
    pub api_key_configured: bool,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiModelProviderUpsertDto {
    pub provider: AiProviderKindDto,
    pub default_model: String,
    pub enabled: bool,
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiProviderTestDto {
    pub provider: AiProviderKindDto,
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiPromptButtonDto {
    pub id: String,
    pub label: String,
    pub target_kind: AiAssetKindDto,
    pub prompt_template: String,
    pub provider: AiProviderKindDto,
    pub model: String,
    pub enabled: bool,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AiPromptButtonUpsertDto {
    pub id: Option<String>,
    pub label: String,
    pub target_kind: AiAssetKindDto,
    pub prompt_template: String,
    pub provider: AiProviderKindDto,
    pub model: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AiAssetsError {
    #[error("{0}")]
    Message(String),
}

impl AiAssetsError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type AiAssetsResult<T> = Result<T, AiAssetsError>;

pub trait AiAssetsApi: 'static {
    fn list_assets(
        &self,
        kind: Option<AiAssetKindDto>,
    ) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiAssetDto>>>;

    fn upsert_asset(
        &self,
        input: AiAssetUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiAssetDto>>;

    fn delete_asset(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>>;

    fn capture(
        &self,
        input: AiCaptureRequestDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiCaptureResponseDto>>;

    fn list_model_providers(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiModelProviderDto>>>;

    fn upsert_model_provider(
        &self,
        input: AiModelProviderUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiModelProviderDto>>;

    fn test_model_provider(
        &self,
        provider: AiProviderKindDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiProviderTestDto>>;

    fn list_prompt_buttons(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiPromptButtonDto>>>;

    fn upsert_prompt_button(
        &self,
        input: AiPromptButtonUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiPromptButtonDto>>;

    fn delete_prompt_button(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>>;
}

pub type SharedAiAssetsApi = Rc<dyn AiAssetsApi>;

pub fn default_ai_assets_api() -> SharedAiAssetsApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserAiAssetsApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedAiAssetsApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserAiAssetsApi;

#[cfg(target_arch = "wasm32")]
impl AiAssetsApi for BrowserAiAssetsApi {
    fn list_assets(
        &self,
        kind: Option<AiAssetKindDto>,
    ) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiAssetDto>>> {
        Box::pin(async move {
            let path = match kind {
                Some(kind) => format!("/api/assets?kind={}", kind.as_str()),
                None => "/api/assets".to_string(),
            };
            super::browser_http::get_json(&path)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn upsert_asset(
        &self,
        input: AiAssetUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiAssetDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/assets/upsert", &input)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn delete_asset(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/assets/{id}"))
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn capture(
        &self,
        input: AiCaptureRequestDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiCaptureResponseDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/ai/run-prompt", &input)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn list_model_providers(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiModelProviderDto>>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/ai/providers")
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn upsert_model_provider(
        &self,
        input: AiModelProviderUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiModelProviderDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/ai/providers/upsert", &input)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn test_model_provider(
        &self,
        provider: AiProviderKindDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiProviderTestDto>> {
        Box::pin(async move {
            let payload = serde_json::json!({ "provider": provider });
            super::browser_http::post_json("/api/ai/providers/test", &payload)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn list_prompt_buttons(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiPromptButtonDto>>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/ai/prompts")
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn upsert_prompt_button(
        &self,
        input: AiPromptButtonUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiPromptButtonDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/ai/prompts/upsert", &input)
                .await
                .map_err(AiAssetsError::new)
        })
    }

    fn delete_prompt_button(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/ai/prompts/{id}"))
                .await
                .map_err(AiAssetsError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedAiAssetsApi;

#[cfg(not(target_arch = "wasm32"))]
impl AiAssetsApi for EmbeddedAiAssetsApi {
    fn list_assets(
        &self,
        kind: Option<AiAssetKindDto>,
    ) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiAssetDto>>> {
        Box::pin(async move { list_assets_on_server(kind).await.map_err(to_service_error) })
    }

    fn upsert_asset(
        &self,
        input: AiAssetUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiAssetDto>> {
        Box::pin(async move {
            upsert_asset_on_server(input)
                .await
                .map_err(to_service_error)
        })
    }

    fn delete_asset(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>> {
        Box::pin(async move { delete_asset_on_server(id).await.map_err(to_service_error) })
    }

    fn capture(
        &self,
        input: AiCaptureRequestDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiCaptureResponseDto>> {
        Box::pin(async move {
            capture_ai_asset_on_server(input)
                .await
                .map_err(to_service_error)
        })
    }

    fn list_model_providers(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiModelProviderDto>>> {
        Box::pin(async move {
            list_model_providers_on_server()
                .await
                .map_err(to_service_error)
        })
    }

    fn upsert_model_provider(
        &self,
        input: AiModelProviderUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiModelProviderDto>> {
        Box::pin(async move {
            upsert_model_provider_on_server(input)
                .await
                .map_err(to_service_error)
        })
    }

    fn test_model_provider(
        &self,
        provider: AiProviderKindDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiProviderTestDto>> {
        Box::pin(async move {
            test_model_provider_on_server(provider)
                .await
                .map_err(to_service_error)
        })
    }

    fn list_prompt_buttons(&self) -> LocalBoxFuture<'_, AiAssetsResult<Vec<AiPromptButtonDto>>> {
        Box::pin(async move {
            list_prompt_buttons_on_server()
                .await
                .map_err(to_service_error)
        })
    }

    fn upsert_prompt_button(
        &self,
        input: AiPromptButtonUpsertDto,
    ) -> LocalBoxFuture<'_, AiAssetsResult<AiPromptButtonDto>> {
        Box::pin(async move {
            upsert_prompt_button_on_server(input)
                .await
                .map_err(to_service_error)
        })
    }

    fn delete_prompt_button(&self, id: String) -> LocalBoxFuture<'_, AiAssetsResult<()>> {
        Box::pin(async move {
            delete_prompt_button_on_server(id)
                .await
                .map_err(to_service_error)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn to_service_error(err: anyhow::Error) -> AiAssetsError {
    AiAssetsError::new(err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_assets_on_server(
    kind: Option<AiAssetKindDto>,
) -> anyhow::Result<Vec<AiAssetDto>> {
    let backend = crate::server::services().await;
    let assets = backend.assets.list_assets(kind.map(Into::into)).await?;
    Ok(assets.into_iter().map(Into::into).collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_asset_on_server(input: AiAssetUpsertDto) -> anyhow::Result<AiAssetDto> {
    let backend = crate::server::services().await;
    let asset = backend.assets.upsert_asset(input.try_into()?).await?;
    Ok(asset.into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_asset_on_server(id: String) -> anyhow::Result<()> {
    let backend = crate::server::services().await;
    backend.assets.delete_asset(parse_uuid(&id)?).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_model_providers_on_server() -> anyhow::Result<Vec<AiModelProviderDto>> {
    use std::collections::BTreeMap;

    let backend = crate::server::services().await;
    let providers = backend.assets.list_providers().await?;
    let mut by_kind = providers
        .into_iter()
        .map(|provider| {
            (
                AiProviderKindDto::from(provider.provider),
                AiModelProviderDto::from(provider),
            )
        })
        .collect::<BTreeMap<_, _>>();

    Ok(AiProviderKindDto::ALL
        .into_iter()
        .map(|provider| {
            by_kind
                .remove(&provider)
                .unwrap_or_else(|| AiModelProviderDto {
                    provider,
                    label: provider.label().to_string(),
                    default_model: addzero_ai_agent::default_model_for(provider.into()).to_string(),
                    enabled: false,
                    api_key_configured: false,
                    updated_at: None,
                })
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_model_provider_on_server(
    input: AiModelProviderUpsertDto,
) -> anyhow::Result<AiModelProviderDto> {
    let backend = crate::server::services().await;
    let provider = backend.assets.upsert_provider(input.into()).await?;
    Ok(provider.into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn test_model_provider_on_server(
    provider: AiProviderKindDto,
) -> anyhow::Result<AiProviderTestDto> {
    let backend = crate::server::services().await;
    let configured = backend.assets.provider_secret(provider.into()).await?;
    let (ok, message) = if configured.is_some() {
        (
            true,
            "配置检查通过：服务端可以解密 API key；真实模型调用会在提示词执行时使用。".to_string(),
        )
    } else {
        (
            false,
            "未找到已启用且带 API key 的配置；请保存 key 后再测试。".to_string(),
        )
    };
    Ok(AiProviderTestDto {
        provider,
        ok,
        message,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_prompt_buttons_on_server() -> anyhow::Result<Vec<AiPromptButtonDto>> {
    let backend = crate::server::services().await;
    let prompts = backend.assets.list_prompt_buttons().await?;
    if prompts.is_empty() {
        return Ok(vec![default_prompt_button().into()]);
    }
    Ok(prompts.into_iter().map(Into::into).collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn upsert_prompt_button_on_server(
    input: AiPromptButtonUpsertDto,
) -> anyhow::Result<AiPromptButtonDto> {
    let backend = crate::server::services().await;
    let prompt = backend
        .assets
        .upsert_prompt_button(input.try_into()?)
        .await?;
    Ok(prompt.into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_prompt_button_on_server(id: String) -> anyhow::Result<()> {
    let backend = crate::server::services().await;
    backend.assets.delete_prompt_button(parse_uuid(&id)?).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn capture_ai_asset_on_server(
    input: AiCaptureRequestDto,
) -> anyhow::Result<AiCaptureResponseDto> {
    use addzero_assets::{AssetEdgeUpsert, AssetKind, AssetUpsert};

    let raw_content = input.raw_content.trim().to_string();
    if raw_content.is_empty() {
        anyhow::bail!("采集内容不能为空");
    }

    let backend = crate::server::services().await;
    let capture = backend
        .assets
        .upsert_asset(AssetUpsert {
            id: None,
            kind: AssetKind::Capture,
            title: capture_title(&raw_content),
            body: raw_content.clone(),
            tags: vec!["采集".to_string()],
            status: "active".to_string(),
            metadata: serde_json::json!({ "source": "admin_capture" }),
        })
        .await?;

    let target_kind = addzero_assets::AssetKind::from(input.target_kind);
    let prompt = select_prompt_for_run(input.prompt_button_id.as_deref(), target_kind).await?;
    let provider_secret = match prompt.as_ref() {
        Some(prompt) => backend.assets.provider_secret(prompt.provider).await?,
        None => None,
    };
    let output = backend
        .asset_agent
        .run_with_provider_secret(&raw_content, target_kind, prompt.as_ref(), provider_secret)
        .await?;

    let generated = backend
        .assets
        .upsert_asset(AssetUpsert {
            id: None,
            kind: target_kind,
            title: output.title.clone(),
            body: output.body.clone(),
            tags: output.tags.clone(),
            status: "active".to_string(),
            metadata: serde_json::json!({
                "source": "ai_capture",
                "capture_asset_id": capture.id,
                "prompt_button_id": prompt.as_ref().map(|prompt| prompt.id),
            }),
        })
        .await?;

    backend
        .assets
        .upsert_edge(AssetEdgeUpsert {
            source_asset_id: capture.id,
            target_asset_id: generated.id,
            relation: "summarizes".to_string(),
            confidence: 1.0,
            metadata: serde_json::json!({ "source": "ai_capture" }),
        })
        .await?;

    for edge in &output.suggested_edges {
        if edge.target_title.trim().is_empty() || edge.target_title == generated.title {
            continue;
        }
        let target = backend
            .assets
            .upsert_asset(AssetUpsert {
                id: None,
                kind: AssetKind::Note,
                title: edge.target_title.clone(),
                body: String::new(),
                tags: vec!["图谱".to_string()],
                status: "suggested".to_string(),
                metadata: serde_json::json!({
                    "source": "agent_suggestion",
                    "placeholder": true,
                }),
            })
            .await?;
        backend
            .assets
            .upsert_edge(AssetEdgeUpsert {
                source_asset_id: generated.id,
                target_asset_id: target.id,
                relation: edge.relation.clone(),
                confidence: f64::from(edge.confidence) / 100.0,
                metadata: serde_json::json!({ "source": "agent_suggestion" }),
            })
            .await?;
    }

    mirror_generated_asset_to_legacy_graph(&generated).await;

    Ok(AiCaptureResponseDto {
        capture_asset: capture.into(),
        generated_asset: generated.into(),
        suggested_edges: output.suggested_edges.into_iter().map(Into::into).collect(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn mirror_skill_asset_on_server(skill: &addzero_skills::Skill) -> anyhow::Result<()> {
    use addzero_assets::{AssetKind, AssetUpsert};

    let backend = crate::server::services().await;
    let existing_id = backend
        .assets
        .list_assets(Some(AssetKind::Skill))
        .await?
        .into_iter()
        .find(|asset| {
            asset
                .metadata
                .get("skill_name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == skill.name)
        })
        .map(|asset| asset.id);

    backend
        .assets
        .upsert_asset(AssetUpsert {
            id: existing_id,
            kind: AssetKind::Skill,
            title: skill.name.clone(),
            body: skill.body.clone(),
            tags: skill.keywords.clone(),
            status: "active".to_string(),
            metadata: serde_json::json!({
                "source": "skill_adapter",
                "skill_name": skill.name,
                "description": skill.description,
                "skill_source": format!("{:?}", skill.source),
            }),
        })
        .await?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_skill_asset_on_server(name: &str) -> anyhow::Result<()> {
    use addzero_assets::AssetKind;

    let backend = crate::server::services().await;
    let ids = backend
        .assets
        .list_assets(Some(AssetKind::Skill))
        .await?
        .into_iter()
        .filter(|asset| {
            asset
                .metadata
                .get("skill_name")
                .and_then(Value::as_str)
                .is_some_and(|skill_name| skill_name == name)
        })
        .map(|asset| asset.id)
        .collect::<Vec<_>>();
    for id in ids {
        backend.assets.delete_asset(id).await?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn select_prompt_for_run(
    prompt_button_id: Option<&str>,
    target_kind: addzero_assets::AssetKind,
) -> anyhow::Result<Option<addzero_assets::AiPromptButton>> {
    let backend = crate::server::services().await;
    let prompts = backend.assets.list_prompt_buttons().await?;
    if let Some(id) = prompt_button_id {
        let parsed = parse_uuid(id)?;
        if parsed == default_prompt_button_id() {
            return Ok(Some(default_prompt_button()));
        }
        if let Some(prompt) = prompts
            .iter()
            .find(|prompt| prompt.id == parsed && prompt.enabled)
            .cloned()
        {
            return Ok(Some(prompt));
        }
    }
    Ok(prompts
        .into_iter()
        .find(|prompt| prompt.enabled && prompt.target_kind == target_kind)
        .or_else(|| Some(default_prompt_button())))
}

#[cfg(not(target_arch = "wasm32"))]
async fn mirror_generated_asset_to_legacy_graph(asset: &addzero_assets::Asset) {
    let Some(kind) = legacy_graph_kind(asset.kind) else {
        return;
    };
    let input = crate::services::asset_graph::AssetRecordInput {
        id: format!("asset:{}", asset.id),
        kind,
        title: asset.title.clone(),
        detail: asset.body.clone(),
        source: "ai_capture".to_string(),
        local_path: None,
        relative_path: None,
        download_url: None,
        content_hash: Some(asset.content_hash.clone()),
        hash_algorithm: Some("sha256".to_string()),
        size_bytes: Some(asset.body.len() as u64),
        tags: asset.tags.clone(),
        raw: serde_json::json!({
            "asset_id": asset.id,
            "metadata": asset.metadata,
        }),
    };
    if let Err(err) = crate::services::asset_graph::upsert_asset_record_on_server(input).await {
        log::warn!("failed to mirror AI asset into legacy graph table: {err}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn legacy_graph_kind(kind: addzero_assets::AssetKind) -> Option<crate::services::AssetKindDto> {
    match kind {
        addzero_assets::AssetKind::Note => Some(crate::services::AssetKindDto::Note),
        addzero_assets::AssetKind::Software => Some(crate::services::AssetKindDto::Software),
        addzero_assets::AssetKind::Package => Some(crate::services::AssetKindDto::Package),
        addzero_assets::AssetKind::Capture | addzero_assets::AssetKind::Skill => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_prompt_button_id() -> uuid::Uuid {
    uuid::Uuid::from_u128(1)
}

#[cfg(not(target_arch = "wasm32"))]
fn default_prompt_button() -> addzero_assets::AiPromptButton {
    addzero_assets::AiPromptButton {
        id: default_prompt_button_id(),
        label: "归纳为笔记".to_string(),
        target_kind: addzero_assets::AssetKind::Note,
        prompt_template:
            "把用户连续输入的采集内容归纳成结构化笔记，自动生成标题、标签、正文摘要和知识图谱关系。"
                .to_string(),
        provider: addzero_assets::AiProviderKind::OpenAi,
        model: addzero_ai_agent::default_model_for(addzero_assets::AiProviderKind::OpenAi)
            .to_string(),
        enabled: true,
        updated_at: chrono::Utc::now(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_title(raw_content: &str) -> String {
    raw_content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("采集")
        .chars()
        .take(36)
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_uuid(id: &str) -> anyhow::Result<uuid::Uuid> {
    uuid::Uuid::parse_str(id).map_err(|err| anyhow::anyhow!("invalid uuid {id}: {err}"))
}

#[cfg(not(target_arch = "wasm32"))]
impl From<AiAssetKindDto> for addzero_assets::AssetKind {
    fn from(value: AiAssetKindDto) -> Self {
        match value {
            AiAssetKindDto::Capture => Self::Capture,
            AiAssetKindDto::Note => Self::Note,
            AiAssetKindDto::Skill => Self::Skill,
            AiAssetKindDto::Software => Self::Software,
            AiAssetKindDto::Package => Self::Package,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::AssetKind> for AiAssetKindDto {
    fn from(value: addzero_assets::AssetKind) -> Self {
        match value {
            addzero_assets::AssetKind::Capture => Self::Capture,
            addzero_assets::AssetKind::Note => Self::Note,
            addzero_assets::AssetKind::Skill => Self::Skill,
            addzero_assets::AssetKind::Software => Self::Software,
            addzero_assets::AssetKind::Package => Self::Package,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<AiProviderKindDto> for addzero_assets::AiProviderKind {
    fn from(value: AiProviderKindDto) -> Self {
        match value {
            AiProviderKindDto::OpenAi => Self::OpenAi,
            AiProviderKindDto::Anthropic => Self::Anthropic,
            AiProviderKindDto::Gemini => Self::Gemini,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::AiProviderKind> for AiProviderKindDto {
    fn from(value: addzero_assets::AiProviderKind) -> Self {
        match value {
            addzero_assets::AiProviderKind::OpenAi => Self::OpenAi,
            addzero_assets::AiProviderKind::Anthropic => Self::Anthropic,
            addzero_assets::AiProviderKind::Gemini => Self::Gemini,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::Asset> for AiAssetDto {
    fn from(value: addzero_assets::Asset) -> Self {
        Self {
            id: value.id.to_string(),
            kind: value.kind.into(),
            title: value.title,
            body: value.body,
            tags: value.tags,
            status: value.status,
            metadata: value.metadata,
            content_hash: value.content_hash,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<AiAssetUpsertDto> for addzero_assets::AssetUpsert {
    type Error = anyhow::Error;

    fn try_from(value: AiAssetUpsertDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.as_deref().map(parse_uuid).transpose()?,
            kind: value.kind.into(),
            title: value.title,
            body: value.body,
            tags: value.tags,
            status: if value.status.trim().is_empty() {
                "active".to_string()
            } else {
                value.status
            },
            metadata: value.metadata,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::AiModelProvider> for AiModelProviderDto {
    fn from(value: addzero_assets::AiModelProvider) -> Self {
        let provider = AiProviderKindDto::from(value.provider);
        Self {
            provider,
            label: provider.label().to_string(),
            default_model: value.default_model,
            enabled: value.enabled,
            api_key_configured: value.api_key_configured,
            updated_at: Some(value.updated_at.to_rfc3339()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<AiModelProviderUpsertDto> for addzero_assets::AiModelProviderUpsert {
    fn from(value: AiModelProviderUpsertDto) -> Self {
        Self {
            provider: value.provider.into(),
            default_model: value.default_model,
            enabled: value.enabled,
            api_key: value.api_key.filter(|key| !key.trim().is_empty()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::AiPromptButton> for AiPromptButtonDto {
    fn from(value: addzero_assets::AiPromptButton) -> Self {
        Self {
            id: value.id.to_string(),
            label: value.label,
            target_kind: value.target_kind.into(),
            prompt_template: value.prompt_template,
            provider: value.provider.into(),
            model: value.model,
            enabled: value.enabled,
            updated_at: Some(value.updated_at.to_rfc3339()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<AiPromptButtonUpsertDto> for addzero_assets::AiPromptButtonUpsert {
    type Error = anyhow::Error;

    fn try_from(value: AiPromptButtonUpsertDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.as_deref().map(parse_uuid).transpose()?,
            label: value.label,
            target_kind: value.target_kind.into(),
            prompt_template: value.prompt_template,
            provider: value.provider.into(),
            model: value.model,
            enabled: value.enabled,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<addzero_assets::SuggestedEdge> for SuggestedEdgeDto {
    fn from(value: addzero_assets::SuggestedEdge) -> Self {
        Self {
            target_title: value.target_title,
            relation: value.relation,
            confidence: value.confidence,
        }
    }
}
