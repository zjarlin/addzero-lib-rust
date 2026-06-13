// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InputImageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageDetail,
};

/// An image input to the model. Learn about [image inputs](/docs/guides/vision).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputImageContent {
    /// The type of the input item. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The detail level of the image to be sent to the model. One of `high`, `low`, `auto`, or `original`.
    /// Defaults to `auto`.
    pub detail: ImageDetail,
}
