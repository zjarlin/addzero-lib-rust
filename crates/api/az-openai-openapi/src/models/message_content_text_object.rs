// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentTextObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentTextObjectText,
};

/// The text content that is part of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentTextObject {
    /// Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub text: MessageContentTextObjectText,
}
