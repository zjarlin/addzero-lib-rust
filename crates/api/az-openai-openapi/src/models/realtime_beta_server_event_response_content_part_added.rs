// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseContentPartAdded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaServerEventResponseContentPartAddedPart,
};

/// Returned when a new content part is added to an assistant message item during response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseContentPartAdded {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.content_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the item to which the content part was added.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The index of the content part in the item's content array.
    pub content_index: i32,
    /// The content part that was added.
    pub part: RealtimeBetaServerEventResponseContentPartAddedPart,
}
