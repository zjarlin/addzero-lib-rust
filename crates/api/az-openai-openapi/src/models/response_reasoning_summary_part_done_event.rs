// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseReasoningSummaryPartDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseReasoningSummaryPartDoneEventPart,
};

/// Emitted when a reasoning summary part is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryPartDoneEvent {
    /// The type of the event. Always `response.reasoning_summary_part.done`.
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
    /// The completed summary part.
    pub part: ResponseReasoningSummaryPartDoneEventPart,
}
