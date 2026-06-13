// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseRefusalDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when refusal text is finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRefusalDoneEvent {
    /// The type of the event. Always `response.refusal.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the refusal text is finalized.
    pub item_id: String,
    /// The index of the output item that the refusal text is finalized.
    pub output_index: i32,
    /// The index of the content part that the refusal text is finalized.
    pub content_index: i32,
    /// The refusal text that is finalized.
    pub refusal: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
