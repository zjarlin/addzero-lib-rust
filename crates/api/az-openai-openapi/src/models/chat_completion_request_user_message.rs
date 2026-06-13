// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestUserMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestUserMessageContent,
};

/// Messages sent by an end user, containing prompts or additional context information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestUserMessage {
    /// The contents of the user message.
    pub content: ChatCompletionRequestUserMessageContent,
    /// The role of the messages author, in this case `user`.
    pub role: String,
    /// An optional name for the participant. Provides the model information to differentiate between
    /// participants of the same role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
