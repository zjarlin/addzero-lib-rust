// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentImageUrlObjectImageUrl` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentImageUrlObjectImageUrl {
    /// The external URL of the image, must be a supported image types: jpeg, jpg, png, gif, webp.
    pub url: String,
    /// Specifies the detail level of the image. `low` uses fewer tokens, you can opt in to high resolution
    /// using `high`. Default value is `auto`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
