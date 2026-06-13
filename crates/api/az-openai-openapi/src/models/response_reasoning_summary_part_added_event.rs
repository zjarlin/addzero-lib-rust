// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseReasoningSummaryPartAddedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseReasoningSummaryPartAddedEventPart,
};

/// Emitted when a new reasoning summary part is added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryPartAddedEvent {
    /// The type of the event. Always `response.reasoning_summary_part.added`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item this summary part is associated with.
    pub item_id: String,
    /// The index of the output item this summary part is associated with.
    pub output_index: i32,
    /// The index of the summary part within the reasoning summary.
    pub summary_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The summary part that was added.
    pub part: ResponseReasoningSummaryPartAddedEventPart,
}
