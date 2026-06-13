// Generated from OpenAPI spec. Do not edit by hand.
//! `ImageGenToolInputImageMask` DTO.

use serde::{Deserialize, Serialize};

/// Optional mask for inpainting. Contains `image_url` (string, optional) and `file_id` (string,
/// optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenToolInputImageMask {
    /// Base64-encoded mask image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// File ID for the mask image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
