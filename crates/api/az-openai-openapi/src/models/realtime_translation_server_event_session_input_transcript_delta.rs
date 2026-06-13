// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionInputTranscriptDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when optional source-language transcript text is available. This event is emitted only when
/// `audio.input.transcription` is configured. Transcript deltas are append-only text fragments. Clients
/// should not insert unconditional spaces between deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionInputTranscriptDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.input_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Append-only source-language transcript text.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<i32>,
}
