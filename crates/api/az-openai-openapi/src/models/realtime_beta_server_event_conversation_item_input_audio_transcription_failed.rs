// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailed` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailedError,
};

/// Returned when input audio transcription is configured, and a transcription request for a user
/// message failed. These events are separate from other `error` events so that the client can identify
/// the related Item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailed {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.input_audio_transcription.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the user message item.
    pub item_id: String,
    /// The index of the content part containing the audio.
    pub content_index: i32,
    /// Details of the transcription error.
    pub error: RealtimeBetaServerEventConversationItemInputAudioTranscriptionFailedError,
}
