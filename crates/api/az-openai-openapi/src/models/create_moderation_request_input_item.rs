// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequestInputItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputItemImageUrl,
};

/// An object describing an image to classify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputItem {
    /// Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputItemImageUrl,
}
