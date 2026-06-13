// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationItemInputAudioTranscriptionSegment` DTO.

use serde::{Deserialize, Serialize};

/// Returned when an input audio transcription segment is identified for an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionSegment {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.input_audio_transcription.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item containing the input audio content.
    pub item_id: String,
    /// The index of the input audio content part within the item.
    pub content_index: i32,
    /// The text for this segment.
    pub text: String,
    /// The segment identifier.
    pub id: String,
    /// The detected speaker label for this segment.
    pub speaker: String,
    /// Start time of the segment in seconds.
    pub start: f64,
    /// End time of the segment in seconds.
    pub end: f64,
}
