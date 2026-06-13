// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateModerationRequestInputItemImageUrl` DTO.

use serde::{Deserialize, Serialize};

/// Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputItemImageUrl {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
