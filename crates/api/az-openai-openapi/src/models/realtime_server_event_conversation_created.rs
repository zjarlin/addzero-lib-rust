// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventConversationCreatedConversation,
};

/// Returned when a conversation is created. Emitted right after session creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The conversation resource.
    pub conversation: RealtimeServerEventConversationCreatedConversation,
}
