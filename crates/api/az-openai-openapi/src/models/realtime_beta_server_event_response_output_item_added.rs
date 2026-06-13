// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventResponseOutputItemAdded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when a new Item is created during Response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseOutputItemAdded {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.output_item.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the Response to which the item belongs.
    pub response_id: String,
    /// The index of the output item in the Response.
    pub output_index: i32,
    pub item: RealtimeConversationItem,
}
