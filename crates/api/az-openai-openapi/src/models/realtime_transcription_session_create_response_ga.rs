// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateResponseGA` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateResponseGAAudio,
};

/// A Realtime transcription session configuration object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGA {
    /// The type of session. Always `transcription` for transcription sessions.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    /// The object type. Always `realtime.transcription_session`.
    pub object: String,
    /// Expiration timestamp for the session, in seconds since epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// Additional fields to include in server outputs. - `item.input_audio_transcription.logprobs`: Include
    /// logprobs for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Configuration for input audio for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeTranscriptionSessionCreateResponseGAAudio>,
}
