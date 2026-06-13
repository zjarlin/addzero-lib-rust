// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventConversationItemRetrieved` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when a conversation item is retrieved with `conversation.item.retrieve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventConversationItemRetrieved {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.retrieved`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub item: RealtimeConversationItem,
}
