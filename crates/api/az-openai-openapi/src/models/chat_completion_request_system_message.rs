// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestSystemMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestSystemMessageContent,
};

/// Developer-provided instructions that the model should follow, regardless of messages sent by the
/// user. With o1 models and newer, use `developer` messages for this purpose instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestSystemMessage {
    /// The contents of the system message.
    pub content: ChatCompletionRequestSystemMessageContent,
    /// The role of the messages author, in this case `system`.
    pub role: String,
    /// An optional name for the participant. Provides the model information to differentiate between
    /// participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
