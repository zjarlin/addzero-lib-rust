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

/// Role for a backend-agnostic chat message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    /// System/developer instruction message.
    System,
    /// User input message.
    User,
    /// Assistant output message.
    Assistant,
}

/// Minimal message shape required by chat-only backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// Message role.
    pub role: ChatRole,
    /// Plain text content.
    pub content: String,
    /// Optional image inputs on user messages.
    pub images: Vec<VisionInput>,
}

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Attaches image inputs to a user message.
    pub fn with_images(mut self, images: Vec<VisionInput>) -> Self {
        self.images = images;
        self
    }
}

/// Request for a plain chat backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    /// Requested model ID.
    pub model: String,
    /// Ordered chat messages.
    pub messages: Vec<ChatMessage>,
}

/// Response from a plain chat backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResponse {
    /// Response object ID, or a backend-generated adapter ID.
    pub id: String,
    /// Model ID returned by the backend.
    pub model: String,
    /// Assistant text.
    pub content: String,
}

/// Adapter point for any text-only conversational model API.
#[async_trait::async_trait]
pub trait ChatBackend: Clone + Send + Sync + 'static {
    /// Sends a plain chat request and returns assistant text.
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;
}

/// OpenAI-compatible chat completion backend.
#[derive(Clone)]
pub struct OpenAiChatBackend {
    client: Client<OpenAIConfig>,
}

impl OpenAiChatBackend {
    /// Creates an OpenAI-compatible chat backend.
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
