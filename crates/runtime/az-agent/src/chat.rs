use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::vision::VisionInput;

/// 与具体后端无关的聊天消息角色。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    /// 系统或开发者指令消息。
    System,
    /// 用户输入消息。
    User,
    /// 助手输出消息。
    Assistant,
}

/// 纯聊天后端需要的最小消息结构。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// 消息角色。
    pub role: ChatRole,
    /// 纯文本内容。
    pub content: String,
    /// 用户消息上的可选图片输入。
    pub images: Vec<VisionInput>,
}

impl ChatMessage {
    /// 创建系统消息。
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// 创建用户消息。
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// 创建助手消息。
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// 为用户消息附加图片输入。
    pub fn with_images(mut self, images: Vec<VisionInput>) -> Self {
        self.images = images;
        self
    }
}

/// 纯聊天后端请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    /// 请求的模型 ID。
    pub model: String,
    /// 按顺序排列的聊天消息。
    pub messages: Vec<ChatMessage>,
}

/// 纯聊天后端响应。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResponse {
    /// 响应对象 ID，或后端生成的适配器 ID。
    pub id: String,
    /// 后端返回的模型 ID。
    pub model: String,
    /// 助手文本。
    pub content: String,
}

/// 任意纯文本对话模型 API 的适配点。
#[async_trait::async_trait]
pub trait ChatBackend: Clone + Send + Sync + 'static {
    /// 发送纯聊天请求并返回助手文本。
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;
}

/// 兼容 OpenAI chat completions 的后端。
#[derive(Clone)]
pub struct OpenAiChatBackend {
    client: Client<OpenAIConfig>,
}

impl OpenAiChatBackend {
    /// 创建兼容 OpenAI 的聊天后端。
    pub fn new(client: Client<OpenAIConfig>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl ChatBackend for OpenAiChatBackend {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        if request
            .messages
            .iter()
            .any(|message| !message.images.is_empty())
        {
            return self.chat_byot(request).await;
        }

        let messages = request
            .messages
            .into_iter()
            .map(to_openai_message)
            .collect::<anyhow::Result<Vec<_>>>()?;
        let response = self
            .client
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model(request.model)
                    .messages(messages)
                    .max_completion_tokens(2048u32)
                    .build()?,
            )
            .await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| anyhow::anyhow!("No content in chat response"))?;
        Ok(ChatResponse {
            id: response.id,
            model: response.model,
            content,
        })
    }
}

impl OpenAiChatBackend {
    async fn chat_byot(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        let request_body = json!({
            "model": request.model,
            "messages": request.messages.into_iter().map(chat_message_json).collect::<Vec<_>>(),
            "max_completion_tokens": 2048,
        });
        let response: TolerantChatResponse = self.client.chat().create_byot(request_body).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| anyhow::anyhow!("No content in chat response"))?;

        Ok(ChatResponse {
            id: response.id,
            model: response.model,
            content,
        })
    }
}

fn to_openai_message(message: ChatMessage) -> anyhow::Result<ChatCompletionRequestMessage> {
    match message.role {
        ChatRole::System => Ok(ChatCompletionRequestSystemMessageArgs::default()
            .content(message.content)
            .build()?
            .into()),
        ChatRole::User => Ok(ChatCompletionRequestUserMessageArgs::default()
            .content(message.content)
            .build()?
            .into()),
        ChatRole::Assistant => Ok(ChatCompletionRequestAssistantMessageArgs::default()
            .content(message.content)
            .build()?
            .into()),
    }
}

fn chat_message_json(message: ChatMessage) -> Value {
    let role = match message.role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    };
    if message.images.is_empty() {
        return json!({
            "role": role,
            "content": message.content,
        });
    }

    json!({
        "role": role,
        "content": user_content_json(message.content, message.images),
    })
}

fn user_content_json(text: String, images: Vec<VisionInput>) -> Vec<Value> {
    if images.is_empty() {
        return vec![json!({
            "type": "text",
            "text": text,
        })];
    }

    let mut parts = vec![json!({
        "type": "text",
        "text": text,
    })];
    parts.extend(images.into_iter().map(|image| image.to_chat_content()));
    parts
}

#[derive(Debug, Clone, Deserialize)]
struct TolerantChatResponse {
    id: String,
    model: String,
    #[serde(default)]
    choices: Vec<TolerantChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct TolerantChatChoice {
    message: TolerantChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct TolerantChatMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vision::VisionDetail;

    #[test]
    fn builds_chat_message_with_image_content_parts() {
        let message = ChatMessage::user("描述图片").with_images(vec![
            VisionInput::image_url("https://example.com/a.png").with_detail(VisionDetail::High),
        ]);

        assert_eq!(
            chat_message_json(message),
            json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": "描述图片"},
                    {"type": "image_url", "image_url": {"url": "https://example.com/a.png", "detail": "high"}}
                ]
            })
        );
    }
}
