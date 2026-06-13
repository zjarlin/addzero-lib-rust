// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventResponseAudioTranscriptDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the model-generated transcription of audio output is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventResponseAudioTranscriptDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.output_audio_transcript.delta`.
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
    /// The transcript delta.
    pub delta: String,
}
