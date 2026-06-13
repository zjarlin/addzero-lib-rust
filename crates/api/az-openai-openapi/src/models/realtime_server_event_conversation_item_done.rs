// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventConversationItemDone` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when a conversation item is finalized. The event will include the full content of the Item
/// except for audio data, which can be retrieved separately with a `conversation.item.retrieve` event
/// if needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: Option<String>,
    pub item: RealtimeConversationItem,
}
