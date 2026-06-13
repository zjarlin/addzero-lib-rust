// Generated from OpenAPI spec. Do not edit by hand.
//! `SpeechAudioDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted for each chunk of audio data generated during speech synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAudioDeltaEvent {
    /// The type of the event. Always `speech.audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A chunk of Base64-encoded audio data.
    pub audio: String,
}
