// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentRefusalObject` DTO.

use serde::{Deserialize, Serialize};

/// The refusal content that is part of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentRefusalObject {
    /// The index of the refusal part in the message.
    pub index: i32,
    /// Always `refusal`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}
