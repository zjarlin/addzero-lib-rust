// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseImageGenCallGeneratingEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an image generation tool call is actively generating an image (intermediate state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseImageGenCallGeneratingEvent {
    /// The type of the event. Always 'response.image_generation_call.generating'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the image generation item being processed.
    pub item_id: String,
    /// The sequence number of the image generation item being processed.
    pub sequence_number: i32,
}
