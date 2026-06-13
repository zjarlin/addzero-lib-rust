// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseRefusalDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when there is a partial refusal text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRefusalDeltaEvent {
    /// The type of the event. Always `response.refusal.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the refusal text is added to.
    pub item_id: String,
    /// The index of the output item that the refusal text is added to.
    pub output_index: i32,
    /// The index of the content part that the refusal text is added to.
    pub content_index: i32,
    /// The refusal text that is added.
    pub delta: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
