// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseReasoningTextDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a reasoning text is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningTextDoneEvent {
    /// The type of the event. Always `response.reasoning_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item this reasoning text is associated with.
    pub item_id: String,
    /// The index of the output item this reasoning text is associated with.
    pub output_index: i32,
    /// The index of the reasoning content part.
    pub content_index: i32,
    /// The full text of the completed reasoning content.
    pub text: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
