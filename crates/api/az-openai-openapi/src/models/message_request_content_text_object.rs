// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageRequestContentTextObject` DTO.

use serde::{Deserialize, Serialize};

/// The text content that is part of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRequestContentTextObject {
    /// Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Text content to be sent to the model
    pub text: String,
}
