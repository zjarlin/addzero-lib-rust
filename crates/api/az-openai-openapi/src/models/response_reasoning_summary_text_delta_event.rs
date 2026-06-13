// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseReasoningSummaryTextDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a delta is added to a reasoning summary text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryTextDeltaEvent {
    /// The type of the event. Always `response.reasoning_summary_text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item this summary text delta is associated with.
    pub item_id: String,
    /// The index of the output item this summary text delta is associated with.
    pub output_index: i32,
    /// The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    /// The text delta that was added to the summary.
    pub delta: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
