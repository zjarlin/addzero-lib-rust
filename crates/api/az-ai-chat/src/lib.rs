//! AI/LLM 提供商的统一聊天接口。
//!
//! 提供通用的 [`ChatClient`] trait 和 [`Message`] 类型，通过统一的抽象层
//! 与 OpenAI 兼容接口、Claude（Anthropic）以及 Google Gemini API 进行交互。
//!
//! # 快速开始
//!
//! ```no_run
//! use az_ai_chat::{ChatClient, Message};
//! use az_ai_chat::openai::OpenAiClient;
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

use anyhow::Result;

automod::dir!(pub "src");

/// 单条消息参与者的角色。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    /// 系统提示词，用于给模型提供指令。
    System,
    /// 用户消息。
    User,
    /// 助手，即模型返回的消息。
    Assistant,
}

impl Role {
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
}

/// 单条聊天消息。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    ) -> Result<ChatResponse>;
}
