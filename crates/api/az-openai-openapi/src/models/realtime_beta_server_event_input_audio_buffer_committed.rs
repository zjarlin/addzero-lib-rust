// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventInputAudioBufferCommitted` DTO.

use serde::{Deserialize, Serialize};

/// Returned when an input audio buffer is committed, either by the client or automatically in server
/// VAD mode. The `item_id` property is the ID of the user message item that will be created, thus a
/// `conversation.item.created` event will also be sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventInputAudioBufferCommitted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `input_audio_buffer.committed`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: Option<String>,
    /// The ID of the user message item that will be created.
    pub item_id: String,
}
