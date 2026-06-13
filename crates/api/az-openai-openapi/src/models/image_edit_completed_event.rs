// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageEditCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImagesUsage,
};

/// Emitted when image editing has completed and the final image is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEditCompletedEvent {
    /// The type of the event. Always `image_edit.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded final edited image data, suitable for rendering as an image.
    pub b64_json: String,
    /// The Unix timestamp when the event was created.
    pub created_at: i64,
    /// The size of the edited image.
    pub size: String,
    /// The quality setting for the edited image.
    pub quality: String,
    /// The background setting for the edited image.
    pub background: String,
    /// The output format for the edited image.
    pub output_format: String,
    pub usage: ImagesUsage,
}
