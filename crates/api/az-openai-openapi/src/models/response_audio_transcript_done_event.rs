// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseAudioTranscriptDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the full audio transcript is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAudioTranscriptDoneEvent {
    /// The type of the event. Always `response.audio.transcript.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
