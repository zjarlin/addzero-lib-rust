// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseReasoningTextDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a delta is added to a reasoning text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningTextDeltaEvent {
    /// The type of the event. Always `response.reasoning_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item this reasoning text delta is associated with.
    pub item_id: String,
    /// The index of the output item this reasoning text delta is associated with.
    pub output_index: i32,
    /// The index of the reasoning content part this delta is associated with.
    pub content_index: i32,
    /// The text delta that was added to the reasoning content.
    pub delta: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
