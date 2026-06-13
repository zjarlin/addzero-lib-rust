// Generated from OpenAPI spec. Do not edit by hand.
//! `Message` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentItem,
    MessagePhase2,
    MessageRole,
    MessageStatus,
};

/// A message to or from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The type of the message. Always set to `message`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the message.
    pub id: String,
    /// The status of item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are
    /// returned via API.
    pub status: MessageStatus,
    /// The role of the message. One of `unknown`, `user`, `assistant`, `system`, `critic`, `discriminator`,
    /// `developer`, or `tool`.
    pub role: MessageRole,
    /// The content of the message
    pub content: Vec<MessageContentItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<MessagePhase2>,
}
