// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateModerationRequestInputArray3ItemImageUrl` DTO.

use serde::{Deserialize, Serialize};

/// Contains either an image URL or a data URL for a base64 encoded image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModerationRequestInputArray3ItemImageUrl {
    /// Either a URL of the image or the base64 encoded image data.
    pub url: String,
}
