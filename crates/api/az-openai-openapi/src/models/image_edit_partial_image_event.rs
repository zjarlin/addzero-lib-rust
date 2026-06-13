// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageEditPartialImageEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a partial image is available during image editing streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEditPartialImageEvent {
    /// The type of the event. Always `image_edit.partial_image`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded partial image data, suitable for rendering as an image.
    pub b64_json: String,
    /// The Unix timestamp when the event was created.
    pub created_at: i64,
    /// The size of the requested edited image.
    pub size: String,
    /// The quality setting for the requested edited image.
    pub quality: String,
    /// The background setting for the requested edited image.
    pub background: String,
    /// The output format for the requested edited image.
    pub output_format: String,
    /// 0-based index for the partial image (streaming).
    pub partial_image_index: i32,
}
