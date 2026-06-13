// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseImageGenCallPartialImageEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a partial image is available during image generation streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseImageGenCallPartialImageEvent {
    /// The type of the event. Always 'response.image_generation_call.partial_image'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the image generation item being processed.
    pub item_id: String,
    /// The sequence number of the image generation item being processed.
    pub sequence_number: i32,
    /// 0-based index for the partial image (backend is 1-based, but this is 0-based for the user).
    pub partial_image_index: i32,
    /// Base64-encoded partial image data, suitable for rendering as an image.
    pub partial_image_b64: String,
}
