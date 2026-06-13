// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemInputAudioTranscriptionDelta` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    LogProbProperties,
};

/// Returned when the text value of an input audio transcription content part is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventConversationItemInputAudioTranscriptionDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.input_audio_transcription.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item.
    pub item_id: String,
    /// The index of the content part in the item's content array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_index: Option<i32>,
    /// The text delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProbProperties>>,
}
