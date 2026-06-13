// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequestInputItem3ObjectImageUrl` DTO.

use serde::{Deserialize, Serialize};

/// Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputItem3ObjectImageUrl {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
