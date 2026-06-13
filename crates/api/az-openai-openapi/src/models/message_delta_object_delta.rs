// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageDeltaObjectDelta` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaObjectDeltaContentItem,
};

/// The delta containing the fields that have changed on the Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaObjectDelta {
    /// The entity that produced the message. One of `user` or `assistant`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The content of the message in array of text and/or images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<MessageDeltaObjectDeltaContentItem>>,
}
