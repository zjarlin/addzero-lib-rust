// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseAudioDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the model-generated audio is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.output_audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the item.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The index of the content part in the item's content array.
    pub content_index: i32,
    /// Base64-encoded audio data delta.
    pub delta: String,
}
