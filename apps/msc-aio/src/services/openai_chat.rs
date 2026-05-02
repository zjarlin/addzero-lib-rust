use std::{fs, future::Future, path::PathBuf, pin::Pin, rc::Rc};

use serde::{Deserialize, Serialize};

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiChatConfigDto {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatRequestDto {
    pub messages: Vec<ChatMessageDto>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponseDto {
    pub message: ChatMessageDto,
}

pub trait OpenAiChatApi: 'static {
    fn load_config(&self) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>>;
    fn save_config(
        &self,
        input: OpenAiChatConfigDto,
    ) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>>;
    fn chat(&self, input: ChatRequestDto) -> LocalBoxFuture<'_, Result<ChatResponseDto, String>>;
}

pub type SharedOpenAiChatApi = Rc<dyn OpenAiChatApi>;

pub fn default_openai_chat_api() -> SharedOpenAiChatApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserOpenAiChatApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedOpenAiChatApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserOpenAiChatApi;

#[cfg(target_arch = "wasm32")]
impl OpenAiChatApi for BrowserOpenAiChatApi {
    fn load_config(&self) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>> {
        Box::pin(async move { super::browser_http::get_json("/api/openai-chat/config").await })
    }

    fn save_config(
        &self,
        input: OpenAiChatConfigDto,
    ) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>> {
        Box::pin(
            async move { super::browser_http::post_json("/api/openai-chat/config", &input).await },
        )
    }

    fn chat(&self, input: ChatRequestDto) -> LocalBoxFuture<'_, Result<ChatResponseDto, String>> {
        Box::pin(
            async move { super::browser_http::post_json("/api/openai-chat/chat", &input).await },
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedOpenAiChatApi;

#[cfg(not(target_arch = "wasm32"))]
impl OpenAiChatApi for EmbeddedOpenAiChatApi {
    fn load_config(&self) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>> {
        Box::pin(async move { load_config_on_server().await })
    }

    fn save_config(
        &self,
        input: OpenAiChatConfigDto,
    ) -> LocalBoxFuture<'_, Result<OpenAiChatConfigDto, String>> {
        Box::pin(async move { save_config_on_server(input).await })
    }

    fn chat(&self, input: ChatRequestDto) -> LocalBoxFuture<'_, Result<ChatResponseDto, String>> {
        Box::pin(async move { chat_on_server(input).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_config_on_server() -> Result<OpenAiChatConfigDto, String> {
    Ok(read_config_file().unwrap_or_default())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn save_config_on_server(
    input: OpenAiChatConfigDto,
) -> Result<OpenAiChatConfigDto, String> {
    let normalized = OpenAiChatConfigDto {
        base_url: input.base_url.trim().trim_end_matches('/').to_string(),
        api_key: input.api_key.trim().to_string(),
        model: input.model.trim().to_string(),
    };
    if normalized.base_url.is_empty() {
        return Err("base_url 不能为空".to_string());
    }
    if normalized.api_key.is_empty() {
        return Err("api_key 不能为空".to_string());
    }
    if normalized.model.is_empty() {
        return Err("model 不能为空".to_string());
    }
    write_config_file(&normalized)?;
    Ok(normalized)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn chat_on_server(input: ChatRequestDto) -> Result<ChatResponseDto, String> {
    let config = read_config_file().ok_or_else(|| {
        "未找到聊天配置，请先在系统设置中保存 base_url / api_key / model".to_string()
    })?;
    if input.messages.is_empty() {
        return Err("消息列表不能为空".to_string());
    }

    #[derive(Serialize)]
    struct UpstreamRequest<'a> {
        model: &'a str,
        messages: &'a [ChatMessageDto],
    }

    #[derive(Deserialize)]
    struct UpstreamResponse {
        choices: Vec<UpstreamChoice>,
    }

    #[derive(Deserialize)]
    struct UpstreamChoice {
        message: ChatMessageDto,
    }

    let client = reqwest::Client::new();
    let response = client
        .post(chat_endpoint(&config.base_url))
        .bearer_auth(&config.api_key)
        .json(&UpstreamRequest {
            model: &config.model,
            messages: &input.messages,
        })
        .send()
        .await
        .map_err(|err| format!("请求模型失败：{err}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("模型接口返回 {status}: {}", summarize(&body)));
    }

    let payload: UpstreamResponse = response
        .json()
        .await
        .map_err(|err| format!("解析模型响应失败：{err}"))?;
    let message = payload
        .choices
        .into_iter()
        .next()
        .map(|item| item.message)
        .ok_or_else(|| "模型响应为空".to_string())?;
    Ok(ChatResponseDto { message })
}

#[cfg(not(target_arch = "wasm32"))]
fn chat_endpoint(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/v1/chat/completions")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .map(|home| home.join(".config/msc-aio/openai-chat.json"))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_config_file() -> Option<OpenAiChatConfigDto> {
    let path = config_path()?;
    let body = fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn write_config_file(config: &OpenAiChatConfigDto) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "无法解析 HOME 目录".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建配置目录失败：{err}"))?;
    }
    let body =
        serde_json::to_string_pretty(config).map_err(|err| format!("编码配置失败：{err}"))?;
    fs::write(path, body).map_err(|err| format!("写入配置失败：{err}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn summarize(body: &str) -> String {
    body.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}
