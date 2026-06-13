// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationItemInputAudioTranscriptionFailedError` DTO.

use serde::{Deserialize, Serialize};

/// Details of the transcription error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionFailedError {
    /// The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// A human-readable error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Parameter related to the error, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}
