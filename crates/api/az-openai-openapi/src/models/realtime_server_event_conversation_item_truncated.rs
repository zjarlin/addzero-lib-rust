// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationItemTruncated` DTO.

use serde::{Deserialize, Serialize};

/// Returned when an earlier assistant audio message item is truncated by the client with a
/// `conversation.item.truncate` event. This event is used to synchronize the server's understanding of
/// the audio with the client's playback. This action will truncate the audio and remove the server-side
/// text transcript to ensure there is no text in the context that hasn't been heard by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemTruncated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.truncated`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the assistant message item that was truncated.
    pub item_id: String,
    /// The index of the content part that was truncated.
    pub content_index: i32,
    /// The duration up to which the audio was truncated, in milliseconds.
    pub audio_end_ms: i32,
}
