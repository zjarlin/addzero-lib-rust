// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionAudio,
};

/// A Realtime translation session. Translation sessions continuously translate input audio into the
/// configured output language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSession {
    /// Unique identifier for the session that looks like `sess_1234567890abcdef`.
    pub id: String,
    /// The session type. Always `translation` for Realtime translation sessions.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Expiration timestamp for the session, in seconds since epoch.
    pub expires_at: i64,
    /// The Realtime translation model used for this session. This field is set at session creation and
    /// cannot be changed with `session.update`.
    pub model: String,
    /// Configuration for translation input and output audio.
    pub audio: RealtimeTranslationSessionAudio,
}
