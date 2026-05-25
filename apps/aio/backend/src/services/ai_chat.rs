use std::collections::BTreeMap;

use az_derive_aliases::{apply, serde_code_default_ord, serde_eq_default};
use rig::{
    client::CompletionClient,
    completion::{AssistantContent, CompletionError, Message},
    providers::{anthropic, gemini},
};
use serde::{Deserialize, Serialize};

#[apply(serde_code_default_ord)]
pub enum AiProviderKindDto {
    #[default]
    OpenAi,
    Anthropic,
    Gemini,
}

impl AiProviderKindDto {
    pub const ALL: [Self; 3] = [Self::OpenAi, Self::Anthropic, Self::Gemini];

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Gemini",
        }
    }
}

#[apply(serde_eq_default)]
pub struct AiProviderConfigDto {
    pub provider: AiProviderKindDto,
    pub label: String,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub api_key_configured: bool,
    pub updated_at: Option<String>,
}

#[apply(serde_eq_default)]
pub struct AiProviderConfigUpsertDto {
    pub provider: AiProviderKindDto,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub api_key: Option<String>,
}

#[apply(serde_eq_default)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

#[apply(serde_eq_default)]
pub struct ChatRequestDto {
    #[serde(default)]
    pub provider: Option<AiProviderKindDto>,
    pub messages: Vec<ChatMessageDto>,
}

#[apply(serde_eq_default)]
pub struct ChatResponseDto {
    pub provider: AiProviderKindDto,
    pub model: String,
    pub message: ChatMessageDto,
}

pub async fn list_provider_configs_on_server() -> Result<Vec<AiProviderConfigDto>, String> {
    let backend = crate::server::services().await;
    let providers = backend
        .assets
        .list_providers()
        .await
        .map_err(|err| err.to_string())?;

    let mut by_kind = providers
        .into_iter()
        .map(|provider| {
            (
                AiProviderKindDto::from(provider.provider),
                AiProviderConfigDto::from(provider),
            )
        })
        .collect::<BTreeMap<_, _>>();

    Ok(AiProviderKindDto::ALL
        .into_iter()
        .map(|provider| {
            by_kind
                .remove(&provider)
                .unwrap_or_else(|| default_provider_config(provider))
        })
        .collect())
}

pub async fn upsert_provider_config_on_server(
    input: AiProviderConfigUpsertDto,
) -> Result<AiProviderConfigDto, String> {
    let normalized_model = input.default_model.trim().to_string();
    if normalized_model.is_empty() {
        return Err("model 不能为空".to_string());
    }
    let normalized_base_url = input
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/').to_string());

    let backend = crate::server::services().await;
    let provider = backend
        .assets
        .upsert_provider(az_assets::AiModelProviderUpsert {
            provider: input.provider.into(),
            base_url: normalized_base_url,
            default_model: normalized_model,
            enabled: input.enabled,
            api_key: input
                .api_key
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
        .await
        .map_err(|err| err.to_string())?;

    Ok(provider.into())
}

pub async fn chat_on_server(input: ChatRequestDto) -> Result<ChatResponseDto, String> {
    let messages = normalize_messages(input.messages)?;
    let provider = resolve_provider(input.provider).await?;
    let backend = crate::server::services().await;
    let secret = backend
        .assets
        .provider_secret(provider.into())
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("{} 尚未启用，或还没有保存 API key。", provider.label()))?;

    match provider {
        AiProviderKindDto::OpenAi => send_openai_compatible_chat(provider, secret, messages).await,
        AiProviderKindDto::Anthropic => {
            let model_name = secret.default_model.clone();
            let client = build_anthropic_client(secret.base_url.as_deref(), &secret.api_key)
                .map_err(|err| format!("初始化 Anthropic client 失败：{err}"))?;
            send_completion(
                provider,
                model_name.clone(),
                client.completion_model(model_name),
                messages,
            )
            .await
        }
        AiProviderKindDto::Gemini => {
            let model_name = secret.default_model.clone();
            let client = build_gemini_client(secret.base_url.as_deref(), &secret.api_key)
                .map_err(|err| format!("初始化 Gemini client 失败：{err}"))?;
            send_completion(
                provider,
                model_name.clone(),
                client.completion_model(model_name),
                messages,
            )
            .await
        }
    }
}

async fn resolve_provider(
    requested: Option<AiProviderKindDto>,
) -> Result<AiProviderKindDto, String> {
    if let Some(provider) = requested {
        return Ok(provider);
    }

    let configs = list_provider_configs_on_server().await?;
    configs
        .into_iter()
        .find(|provider| provider.enabled && provider.api_key_configured)
        .map(|provider| provider.provider)
        .ok_or_else(|| "没有可用的 AI provider，请先在环境与配置页启用并配置 API key。".to_string())
}

fn normalize_messages(messages: Vec<ChatMessageDto>) -> Result<Vec<Message>, String> {
    let normalized = messages
        .into_iter()
        .filter_map(|message| {
            let content = message.content.trim().to_string();
            if content.is_empty() {
                return None;
            }
            Some((message.role.trim().to_lowercase(), content))
        })
        .map(|(role, content)| match role.as_str() {
            "system" => Ok(Message::system(content)),
            "user" => Ok(Message::user(content)),
            "assistant" => Ok(Message::assistant(content)),
            other => Err(format!("不支持的消息角色：{other}")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if normalized.is_empty() {
        return Err("消息列表不能为空".to_string());
    }

    Ok(normalized)
}

async fn send_completion<M>(
    provider: AiProviderKindDto,
    model_name: String,
    model: M,
    mut messages: Vec<Message>,
) -> Result<ChatResponseDto, String>
where
    M: rig::completion::CompletionModel + Clone,
{
    let prompt = messages
        .pop()
        .ok_or_else(|| "消息列表不能为空".to_string())?;
    let mut builder = model.completion_request(prompt).messages(messages);

    if provider == AiProviderKindDto::Anthropic {
        builder = builder.max_tokens(4096);
    }

    let response = builder
        .send()
        .await
        .map_err(|err| format!("请求模型失败：{}", summarize_completion_error(&err)))?;
    let content = extract_text(&response.choice);
    if content.trim().is_empty() {
        return Err("模型响应为空".to_string());
    }

    Ok(ChatResponseDto {
        provider,
        model: model_name,
        message: ChatMessageDto {
            role: "assistant".to_string(),
            content,
        },
    })
}

async fn send_openai_compatible_chat(
    provider: AiProviderKindDto,
    secret: az_assets::AssetProviderSecret,
    messages: Vec<Message>,
) -> Result<ChatResponseDto, String> {
    #[derive(Serialize)]
    struct OpenAiRequest<'a> {
        model: &'a str,
        messages: Vec<OpenAiMessage<'a>>,
    }

    #[derive(Serialize)]
    struct OpenAiMessage<'a> {
        role: &'a str,
        content: &'a str,
    }

    #[derive(Deserialize)]
    struct OpenAiResponse {
        model: Option<String>,
        choices: Vec<OpenAiChoice>,
    }

    #[derive(Deserialize)]
    struct OpenAiChoice {
        message: OpenAiChoiceMessage,
    }

    #[derive(Deserialize)]
    struct OpenAiChoiceMessage {
        content: String,
    }

    let base_url = normalize_openai_base_url(secret.base_url.as_deref());
    let request_messages = messages
        .iter()
        .map(|message| match message {
            Message::System { content } => OpenAiMessage {
                role: "system",
                content,
            },
            Message::User { content } => OpenAiMessage {
                role: "user",
                content: content
                    .iter()
                    .find_map(|item| match item {
                        rig::completion::message::UserContent::Text(text) => Some(text.text()),
                        _ => None,
                    })
                    .unwrap_or_default(),
            },
            Message::Assistant { content, .. } => OpenAiMessage {
                role: "assistant",
                content: content
                    .iter()
                    .find_map(|item| match item {
                        AssistantContent::Text(text) => Some(text.text()),
                        AssistantContent::Reasoning(reasoning) => reasoning.first_text(),
                        _ => None,
                    })
                    .unwrap_or_default(),
            },
        })
        .collect::<Vec<_>>();

    let response = reqwest::Client::new()
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(&secret.api_key)
        .json(&OpenAiRequest {
            model: &secret.default_model,
            messages: request_messages,
        })
        .send()
        .await
        .map_err(|err| format!("请求模型失败：{err}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("读取模型响应失败：{err}"))?;
    if !status.is_success() {
        return Err(format!("模型接口返回 {status}: {}", summarize_text(&body)));
    }
    let payload: OpenAiResponse =
        serde_json::from_str(&body).map_err(|err| format!("解析模型响应失败：{err}"))?;
    let content = payload
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "模型响应为空".to_string())?;

    Ok(ChatResponseDto {
        provider,
        model: payload.model.unwrap_or(secret.default_model),
        message: ChatMessageDto {
            role: "assistant".to_string(),
            content,
        },
    })
}

fn extract_text(choice: &rig::OneOrMany<AssistantContent>) -> String {
    choice
        .iter()
        .filter_map(|item| match item {
            AssistantContent::Text(text) => Some(text.text().to_string()),
            AssistantContent::Reasoning(reasoning) => {
                let text = reasoning.display_text();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn summarize_completion_error(err: &CompletionError) -> String {
    let message = err.to_string();
    let collapsed = message.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(240).collect()
}

fn normalize_openai_base_url(base_url: Option<&str>) -> String {
    let trimmed = base_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

fn summarize_text(body: &str) -> String {
    body.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}

fn build_anthropic_client(
    base_url: Option<&str>,
    api_key: &str,
) -> Result<anthropic::Client, rig::client::ProviderClientError> {
    let mut builder = anthropic::Client::builder().api_key(api_key);
    if let Some(base_url) = base_url.filter(|value| !value.trim().is_empty()) {
        builder = builder.base_url(base_url);
    }
    builder.build().map_err(Into::into)
}

fn build_gemini_client(
    base_url: Option<&str>,
    api_key: &str,
) -> Result<gemini::Client, rig::client::ProviderClientError> {
    let mut builder = gemini::Client::builder().api_key(api_key);
    if let Some(base_url) = base_url.filter(|value| !value.trim().is_empty()) {
        builder = builder.base_url(base_url);
    }
    builder.build().map_err(Into::into)
}

fn default_provider_config(provider: AiProviderKindDto) -> AiProviderConfigDto {
    AiProviderConfigDto {
        provider,
        label: provider.label().to_string(),
        base_url: None,
        default_model: az_ai_agent::default_model_for(provider.into()).to_string(),
        enabled: false,
        api_key_configured: false,
        updated_at: None,
    }
}

impl From<AiProviderKindDto> for az_assets::AiProviderKind {
    fn from(value: AiProviderKindDto) -> Self {
        match value {
            AiProviderKindDto::OpenAi => Self::OpenAi,
            AiProviderKindDto::Anthropic => Self::Anthropic,
            AiProviderKindDto::Gemini => Self::Gemini,
        }
    }
}

impl From<az_assets::AiProviderKind> for AiProviderKindDto {
    fn from(value: az_assets::AiProviderKind) -> Self {
        match value {
            az_assets::AiProviderKind::OpenAi => Self::OpenAi,
            az_assets::AiProviderKind::Anthropic => Self::Anthropic,
            az_assets::AiProviderKind::Gemini => Self::Gemini,
        }
    }
}

impl From<az_assets::AiModelProvider> for AiProviderConfigDto {
    fn from(value: az_assets::AiModelProvider) -> Self {
        let provider = AiProviderKindDto::from(value.provider);
        Self {
            provider,
            label: provider.label().to_string(),
            base_url: value.base_url,
            default_model: value.default_model,
            enabled: value.enabled,
            api_key_configured: value.api_key_configured,
            updated_at: Some(value.updated_at.to_rfc3339()),
        }
    }
}
