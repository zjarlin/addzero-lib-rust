// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseImageGenCallInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an image generation tool call is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseImageGenCallInProgressEvent {
    /// The type of the event. Always 'response.image_generation_call.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the image generation item being processed.
    pub item_id: String,
    /// The sequence number of the image generation item being processed.
    pub sequence_number: i32,
}
