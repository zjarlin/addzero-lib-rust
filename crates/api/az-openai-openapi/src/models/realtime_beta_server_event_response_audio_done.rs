// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventResponseAudioDone` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the model-generated audio is done. Also emitted when a Response is interrupted,
/// incomplete, or cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseAudioDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.output_audio.done`.
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
}
