// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaClientEventConversationItemDelete` DTO.

use serde::{Deserialize, Serialize};

/// Send this event when you want to remove any item from the conversation history. The server will
/// respond with a `conversation.item.deleted` event, unless the item does not exist in the conversation
/// history, in which case the server will respond with an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventConversationItemDelete {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `conversation.item.delete`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item to delete.
    pub item_id: String,
}
