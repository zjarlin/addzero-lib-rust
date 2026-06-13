// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestDeveloperMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestDeveloperMessageContent,
};

/// Developer-provided instructions that the model should follow, regardless of messages sent by the
/// user. With o1 models and newer, `developer` messages replace the previous `system` messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestDeveloperMessage {
    /// The contents of the developer message.
    pub content: ChatCompletionRequestDeveloperMessageContent,
    /// The role of the messages author, in this case `developer`.
    pub role: String,
    /// An optional name for the participant. Provides the model information to differentiate between
    /// participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
