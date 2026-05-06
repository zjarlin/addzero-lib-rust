//! AI Vibe Coding 编排引擎。
//!
//! 对接大模型（OpenAI / Ollama / 通义），管理提示词模板、角色库、
//! 上下文会话，实现自然语言→脚本/任务/配置文件/工程模板生成。

use serde::{Deserialize, Serialize};

/// Supported AI provider backends.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiProvider {
    OpenAI {
        api_key: String,
        model: String,
    },
    Ollama {
        host: String,
        model: String,
    },
    Custom {
        endpoint: String,
        api_key: String,
        model: String,
    },
}

/// A chat message in a conversation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

/// Input to a vibe-coding generation request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VibeRequest {
    /// Natural language description of what to generate.
    pub prompt: String,
    /// Target output kind.
    pub target: VibeTarget,
    /// Conversation context (previous messages).
    pub context: Vec<AiMessage>,
    /// System prompt override.
    pub system_prompt: Option<String>,
}

/// What kind of output to generate.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VibeTarget {
    /// Generate a script.
    Script { lang: String },
    /// Generate a task flow definition.
    TaskFlow,
    /// Generate a configuration file.
    Config { format: String },
    /// Generate a CLI project scaffold.
    CliProject { template: String },
}

/// Output from a vibe-coding generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VibeResponse {
    /// Generated content.
    pub content: String,
    /// Tokens used.
    pub tokens_used: u64,
    /// The provider that fulfilled the request.
    pub provider: String,
}

/// AI engine trait — pluggable via WASM.
pub trait AiEngine: Send + Sync {
    /// Generate content based on a natural language prompt.
    fn vibe(&self, request: VibeRequest) -> Result<VibeResponse, AiError>;
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AiError {
    #[error("provider error: {0}")]
    Provider(String),
    #[error("context too long: {0} tokens")]
    ContextTooLong(u64),
    #[error("{0}")]
    Other(String),
}
