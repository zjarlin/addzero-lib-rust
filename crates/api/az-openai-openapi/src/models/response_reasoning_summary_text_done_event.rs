// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseReasoningSummaryTextDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a reasoning summary text is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryTextDoneEvent {
    /// The type of the event. Always `response.reasoning_summary_text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item this summary text is associated with.
    pub item_id: String,
    /// The index of the output item this summary text is associated with.
    pub output_index: i32,
    /// The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    /// The full text of the completed reasoning summary.
    pub text: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
