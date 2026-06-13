// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InputImageContentParamAutoParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    DetailEnum,
};

/// An image input to the model. Learn about [image inputs](/docs/guides/vision)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputImageContentParamAutoParam {
    /// The type of the input item. Always `input_image`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailEnum>,
}
