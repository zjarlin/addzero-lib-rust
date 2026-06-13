// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseImageGenCallCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an image generation tool call has completed and the final image is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseImageGenCallCompletedEvent {
    /// The type of the event. Always 'response.image_generation_call.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The unique identifier of the image generation item being processed.
    pub item_id: String,
}
