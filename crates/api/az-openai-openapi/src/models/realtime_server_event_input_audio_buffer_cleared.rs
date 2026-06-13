// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventInputAudioBufferCleared` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the input audio buffer is cleared by the client with a `input_audio_buffer.clear`
/// event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventInputAudioBufferCleared {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `input_audio_buffer.cleared`.
    #[serde(rename = "type")]
    pub type_value: String,
}
