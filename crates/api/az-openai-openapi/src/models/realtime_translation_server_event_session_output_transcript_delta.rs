// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionOutputTranscriptDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when translated transcript text is available. Transcript deltas are append-only text
/// fragments. Clients should not insert unconditional spaces between deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionOutputTranscriptDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.output_transcript.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Append-only transcript text for the translated output audio.
    pub delta: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<i32>,
}
