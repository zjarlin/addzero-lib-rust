// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventConversationCreatedConversation` DTO.

use serde::{Deserialize, Serialize};

/// The conversation resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationCreatedConversation {
    /// The unique ID of the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The object type, must be `realtime.conversation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}
