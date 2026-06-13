// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageRefParam` DTO.

use serde::{Deserialize, Serialize};

/// Reference an input image by either URL or uploaded file ID. Provide exactly one of `image_url` or
/// `file_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRefParam {
    /// A fully qualified URL or base64-encoded data URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// The File API ID of an uploaded image to use as input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
