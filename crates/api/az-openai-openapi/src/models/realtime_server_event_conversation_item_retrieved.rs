// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationItemRetrieved` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when a conversation item is retrieved with `conversation.item.retrieve`. This is provided
/// as a way to fetch the server's representation of an item, for example to get access to the post-
/// processed audio data after noise cancellation and VAD. It includes the full content of the Item,
/// including audio data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemRetrieved {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.retrieved`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub item: RealtimeConversationItem,
}
