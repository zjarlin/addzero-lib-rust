// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OutputMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessagePhase,
    OutputMessageContent,
};

/// An output message from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMessage {
    /// The unique ID of the output message.
    pub id: String,
    /// The type of the output message. Always `message`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The role of the output message. Always `assistant`.
    pub role: String,
    /// The content of the output message.
    pub content: Vec<OutputMessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<MessagePhase>,
    /// The status of the message input. One of `in_progress`, `completed`, or `incomplete`. Populated when
    /// input items are returned via API.
    pub status: String,
}
