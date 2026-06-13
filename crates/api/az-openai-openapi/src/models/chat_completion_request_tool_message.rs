// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestToolMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestToolMessageContent,
};

/// Tool message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestToolMessage {
    /// The role of the messages author, in this case `tool`.
    pub role: String,
    /// The contents of the tool message.
    pub content: ChatCompletionRequestToolMessageContent,
    /// Tool call that this message is responding to.
    pub tool_call_id: String,
}
