// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeClientEventConversationItemCreate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Add a new Item to the Conversation's context, including messages, function calls, and function call
/// responses. This event can be used both to populate a "history" of the conversation and to add new
/// items mid-stream, but has the current limitation that it cannot populate assistant audio messages.
/// If successful, the server will respond with a `conversation.item.created` event, otherwise an
/// `error` event will be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeClientEventConversationItemCreate {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `conversation.item.create`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the preceding item after which the new item will be inserted. If not set, the new item
    /// will be appended to the end of the conversation. If set to `root`, the new item will be added to the
    /// beginning of the conversation. If set to an existing ID, it allows an item to be inserted mid-
    /// conversation. If the ID cannot be found, an error will be returned and the item will not be added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: Option<String>,
    pub item: RealtimeConversationItem,
}
