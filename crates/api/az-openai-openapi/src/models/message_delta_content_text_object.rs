// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageDeltaContentTextObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentTextObjectText,
};

/// The text content that is part of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextObject {
    /// The index of the content part in the message.
    pub index: i32,
    /// Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<MessageDeltaContentTextObjectText>,
}
