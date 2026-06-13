// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when a conversation item is created. There are several scenarios that produce this event: -
/// The server is generating a Response, which if successful will produce either one or two Items, which
/// will be of type `message` (role `assistant`) or type `function_call`. - The input audio buffer has
/// been committed, either by the client or the server (in `server_vad` mode). The server will take the
/// content of the input audio buffer and add it to a new user message Item. - The client has sent a
/// `conversation.item.create` event to add a new Item to the Conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventConversationItemCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_item_id: Option<String>,
    pub item: RealtimeConversationItem,
}
