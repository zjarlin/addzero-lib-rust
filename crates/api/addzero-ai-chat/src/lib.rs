//! Unified Chat interface for AI/LLM providers.
//!
//! Provides a common [`ChatClient`] trait and [`Message`] types to interact
//! with OpenAI-compatible, Claude (Anthropic), and Google Gemini APIs through
//! a single abstraction.
//!
//! # Quick Start
//!
//! ```no_run
//! use addzero_ai_chat::{OpenAiClient, ChatClient, Message, ChatOptions, Role};
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

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod openai;

pub use openai::OpenAiClient;

/// Errors that can occur during chat operations.
#[derive(Debug, Error)]
pub enum ChatError {
    /// HTTP request failed.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The provider returned an error response.
    #[error("provider error ({code}): {message}")]
    ProviderError { code: u16, message: String },

    /// A required field was missing from the response.
    #[error("missing field in response: {0}")]
    MissingField(String),

    /// Invalid configuration (e.g., empty API key).
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

/// Result alias for chat operations.
pub type ChatResult<T> = Result<T, ChatError>;

/// The role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt (instructions to the model).
    System,
    /// User message.
    User,
    /// Assistant (model) response.
    Assistant,
}

/// A single chat message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The text content of the message.
    pub content: String,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Optional parameters for chat completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatOptions {
    /// Sampling temperature (0.0–2.0). Higher = more random.
    pub temperature: Option<f64>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Top-p nucleus sampling.
    pub top_p: Option<f64>,
    /// Stop sequences.
    pub stop: Option<Vec<String>>,
}

impl ChatOptions {
    /// Create default (empty) options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the temperature.
    pub fn with_temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set the max tokens.
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// The response from a chat completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The model's text reply.
    pub content: String,
    /// The model that was used (may differ from request).
    pub model: String,
    /// Token usage statistics, if available.
    pub usage: Option<Usage>,
    /// The finish reason (e.g., "stop", "length").
    pub finish_reason: Option<String>,
}

/// Token usage statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// Number of tokens in the completion.
    pub completion_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
}

/// Trait implemented by all AI chat providers.
///
/// Each provider translates the common [`Message`] / [`ChatOptions`] into its
/// own API format and parses the response back into [`ChatResponse`].
#[async_trait::async_trait]
pub trait ChatClient: Send + Sync {
    /// Send a list of messages and receive a completion.
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
