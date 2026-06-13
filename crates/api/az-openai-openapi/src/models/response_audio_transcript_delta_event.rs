// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseAudioTranscriptDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when there is a partial transcript of audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAudioTranscriptDeltaEvent {
    /// The type of the event. Always `response.audio.transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The partial transcript of the audio response.
    pub delta: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
