// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationRequestInputArray3Item` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputArray3ItemImageUrl,
};

/// An object describing an image to classify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputArray3Item {
    /// Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputArray3ItemImageUrl,
}
