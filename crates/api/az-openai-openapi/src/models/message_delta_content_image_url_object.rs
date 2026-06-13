// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentImageUrlObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentImageUrlObjectImageUrl,
};

/// References an image URL in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentImageUrlObject {
    /// The index of the content part in the message.
    pub index: i32,
    /// Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<MessageDeltaContentImageUrlObjectImageUrl>,
}
