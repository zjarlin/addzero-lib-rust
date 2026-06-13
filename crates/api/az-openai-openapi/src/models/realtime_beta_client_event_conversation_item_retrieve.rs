// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaClientEventConversationItemRetrieve` DTO.

use serde::{Deserialize, Serialize};

/// Send this event when you want to retrieve the server's representation of a specific item in the
/// conversation history. This is useful, for example, to inspect user audio after noise cancellation
/// and VAD. The server will respond with a `conversation.item.retrieved` event, unless the item does
/// not exist in the conversation history, in which case the server will respond with an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventConversationItemRetrieve {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `conversation.item.retrieve`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item to retrieve.
    pub item_id: String,
}
