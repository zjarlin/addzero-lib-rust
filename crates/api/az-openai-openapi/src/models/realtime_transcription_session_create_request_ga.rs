// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateRequestGA` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateRequestGAAudio,
};

/// Realtime transcription session object configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGA {
    /// The type of session to create. Always `transcription` for transcription sessions.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Configuration for input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeTranscriptionSessionCreateRequestGAAudio>,
    /// Additional fields to include in server outputs. `item.input_audio_transcription.logprobs`: Include
    /// logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}
