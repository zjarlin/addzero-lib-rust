//! AI/LLM 提供商的统一聊天接口。
//!
//! 提供通用的 [`ChatClient`] trait 和 [`Message`] 类型，通过统一的抽象层
//! 与 OpenAI 兼容接口、Claude（Anthropic）以及 Google Gemini API 进行交互。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_ai_chat::{OpenAiClient, ChatClient, Message, ChatOptions, Role};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = OpenAiClient::new("https://api.openai.com/v1", "sk-...");
//! let messages = vec![
//!     Message::system("You are a helpful assistant."),
//!     Message::user("Hello!"),
//! ];
//! let reply = client.chat("gpt-4", &messages, None).await?;
//! println!("{}", reply.content);
//! # Ok(())
//! # }
//! ```

use az_derive_aliases::{apply, error, serde_code_enum, serde_eq, serde_partial_eq_default};

automod::dir!("src");

pub use openai::OpenAiClient;

/// 聊天调用过程中可能出现的错误。
#[apply(error)]
pub enum ChatError {
    /// HTTP 请求失败。
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 序列化或反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 模型供应商返回错误响应。
    #[error("provider error ({code}): {message}")]
    ProviderError { code: u16, message: String },

    /// 响应中缺少必需字段。
    #[error("missing field in response: {0}")]
    MissingField(String),

    /// 配置不合法，例如 API key 为空。
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

/// 聊天操作的统一结果类型。
pub type ChatResult<T> = Result<T, ChatError>;

/// 单条消息参与者的角色。
#[apply(serde_code_enum)]
pub enum Role {
    /// 系统提示词，用于给模型提供指令。
    System,
    /// 用户消息。
    User,
    /// 助手，即模型返回的消息。
    Assistant,
}

/// 单条聊天消息。
#[apply(serde_eq)]
pub struct Message {
    /// 消息发送者角色。
    pub role: Role,
    /// 消息文本内容。
    pub content: String,
}

impl Message {
    /// 创建系统消息。
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// 创建用户消息。
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// 创建助手消息。
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// 聊天补全的可选参数。
#[apply(serde_partial_eq_default)]
pub struct ChatOptions {
    /// 采样温度，通常为 `0.0..=2.0`；值越高随机性越强。
    pub temperature: Option<f64>,
    /// 最大生成 token 数。
    pub max_tokens: Option<u32>,
    /// nucleus sampling 的 top-p 参数。
    pub top_p: Option<f64>,
    /// 停止序列。
    pub stop: Option<Vec<String>>,
}

impl ChatOptions {
    /// 创建空的默认参数。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置采样温度。
    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// 设置最大生成 token 数。
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// 聊天补全响应。
#[apply(serde_eq)]
pub struct ChatResponse {
    /// 模型返回的文本。
    pub content: String,
    /// 实际使用的模型，可能不同于请求中的模型名。
    pub model: String,
    /// token 用量统计；供应商返回时才有值。
    pub usage: Option<Usage>,
    /// 结束原因，例如 `stop` 或 `length`。
    pub finish_reason: Option<String>,
}

/// token 用量统计。
#[apply(serde_eq)]
pub struct Usage {
    /// prompt token 数。
    pub prompt_tokens: u32,
    /// completion token 数。
    pub completion_tokens: u32,
    /// 总 token 数。
    pub total_tokens: u32,
}

/// 所有 AI 聊天供应商共同实现的 trait。
///
/// 每个供应商负责将通用 [`Message`] / [`ChatOptions`] 转换为自己的 API 格式，
/// 再把响应解析回 [`ChatResponse`]。
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    /// 发送消息列表并接收一次补全响应。
    async fn chat(
        &self,
        model: &str,
        messages: &[Message],
        options: Option<&ChatOptions>,
    ) -> ChatResult<ChatResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_constructors() {
        let sys = Message::system("be helpful");
        assert_eq!(sys.role, Role::System);
        assert_eq!(sys.content, "be helpful");

        let usr = Message::user("hello");
        assert_eq!(usr.role, Role::User);

        let asst = Message::assistant("hi there");
        assert_eq!(asst.role, Role::Assistant);
    }

    #[test]
    fn chat_options_builder() {
        let opts = ChatOptions::new()
            .with_temperature(0.7)
            .with_max_tokens(256);
        assert_eq!(opts.temperature, Some(0.7));
        assert_eq!(opts.max_tokens, Some(256));
        assert!(opts.stop.is_none());
    }

    #[test]
    fn message_serialization_roundtrip() {
        let msg = Message::user("test message");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn role_serialization() {
        let json = serde_json::to_string(&Role::System).unwrap();
        assert_eq!(json, "\"system\"");
        let json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(json, "\"user\"");
        let json = serde_json::to_string(&Role::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn chat_response_deserialization() {
        let json = r#"{
            "content": "Hello!",
            "model": "gpt-4",
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15},
            "finish_reason": "stop"
        }"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.model, "gpt-4");
        assert_eq!(resp.usage.unwrap().total_tokens, 15);
        assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn chat_error_display() {
        let err = ChatError::InvalidConfig("empty api key".into());
        assert_eq!(err.to_string(), "invalid config: empty api key");

        let err = ChatError::ProviderError {
            code: 429,
            message: "rate limited".into(),
        };
        assert!(err.to_string().contains("429"));
    }
}
