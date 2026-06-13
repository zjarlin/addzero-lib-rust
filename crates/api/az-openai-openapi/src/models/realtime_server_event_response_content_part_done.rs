// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventResponseContentPartDone` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventResponseContentPartDonePart,
};

/// Returned when a content part is done streaming in an assistant message item. Also emitted when a
/// Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventResponseContentPartDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.content_part.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the item.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The index of the content part in the item's content array.
    pub content_index: i32,
    /// The content part that is done.
    pub part: RealtimeServerEventResponseContentPartDonePart,
}
