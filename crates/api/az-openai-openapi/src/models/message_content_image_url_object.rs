// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageContentImageUrlObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentImageUrlObjectImageUrl,
};

/// References an image URL in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentImageUrlObject {
    /// The type of the content part.
    #[serde(rename = "type")]
    pub type_value: String,
    pub image_url: MessageContentImageUrlObjectImageUrl,
}
