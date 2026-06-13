// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageGenCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImagesUsage,
};

/// Emitted when image generation has completed and the final image is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenCompletedEvent {
    /// The type of the event. Always `image_generation.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded image data, suitable for rendering as an image.
    pub b64_json: String,
    /// The Unix timestamp when the event was created.
    pub created_at: i64,
    /// The size of the generated image.
    pub size: String,
    /// The quality setting for the generated image.
    pub quality: String,
    /// The background setting for the generated image.
    pub background: String,
    /// The output format for the generated image.
    pub output_format: String,
    pub usage: ImagesUsage,
}
