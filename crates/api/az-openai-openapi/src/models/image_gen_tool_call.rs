// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageGenToolCall` DTO.

use serde::{Deserialize, Serialize};

/// An image generation request made by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenToolCall {
    /// The type of the image generation call. Always `image_generation_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the image generation call.
    pub id: String,
    /// The status of the image generation call.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}
