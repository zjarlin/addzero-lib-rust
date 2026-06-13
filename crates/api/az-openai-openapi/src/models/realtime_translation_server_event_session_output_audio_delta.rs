// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionOutputAudioDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when translated output audio is available. Output audio deltas are 200 ms frames of PCM16
/// audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionOutputAudioDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.output_audio.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded translated audio data.
    pub delta: String,
    /// Sample rate of the audio delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    /// Number of audio channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<i32>,
    /// Audio encoding for `delta`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<i32>,
}
