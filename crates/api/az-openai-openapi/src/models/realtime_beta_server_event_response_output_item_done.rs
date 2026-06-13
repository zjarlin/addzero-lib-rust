// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseOutputItemDone` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeConversationItem,
};

/// Returned when an Item is done streaming. Also emitted when a Response is interrupted, incomplete, or
/// cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseOutputItemDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.output_item.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the Response to which the item belongs.
    pub response_id: String,
    /// The index of the output item in the Response.
    pub output_index: i32,
    pub item: RealtimeConversationItem,
}
