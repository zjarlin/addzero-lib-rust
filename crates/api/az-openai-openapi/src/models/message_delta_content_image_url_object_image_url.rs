// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageDeltaContentImageUrlObjectImageUrl` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentImageUrlObjectImageUrl {
    /// The URL of the image, must be a supported image types: jpeg, jpg, png, gif, webp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Specifies the detail level of the image. `low` uses fewer tokens, you can opt in to high resolution
    /// using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
