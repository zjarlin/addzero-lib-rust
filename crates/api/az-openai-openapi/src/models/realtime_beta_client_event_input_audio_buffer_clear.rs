// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaClientEventInputAudioBufferClear` DTO.

use serde::{Deserialize, Serialize};

/// Send this event to clear the audio bytes in the buffer. The server will respond with an
/// `input_audio_buffer.cleared` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventInputAudioBufferClear {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `input_audio_buffer.clear`.
    #[serde(rename = "type")]
    pub type_value: String,
}
