// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemDeleted` DTO.

use serde::{Deserialize, Serialize};

/// Returned when an item in the conversation is deleted by the client with a `conversation.item.delete`
/// event. This event is used to synchronize the server's understanding of the conversation history with
/// the client's view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventConversationItemDeleted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.deleted`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item that was deleted.
    pub item_id: String,
}
