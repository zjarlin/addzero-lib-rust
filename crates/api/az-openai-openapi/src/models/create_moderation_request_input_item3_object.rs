// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationRequestInputItem3Object` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateModerationRequestInputItem3ObjectImageUrl,
};

/// An object describing an image to classify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputItem3Object {
    /// Always `image_url`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Contains either an image URL or a data URL for a base64 encoded image.
    pub image_url: CreateModerationRequestInputItem3ObjectImageUrl,
}
