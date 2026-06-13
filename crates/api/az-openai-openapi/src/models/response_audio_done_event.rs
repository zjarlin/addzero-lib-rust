// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseAudioDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the audio response is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAudioDoneEvent {
    /// The type of the event. Always `response.audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of the delta.
    pub sequence_number: i32,
}
