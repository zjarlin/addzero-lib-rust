// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseCustomToolCallInputDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Event representing a delta (partial update) to the input of a custom tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCustomToolCallInputDeltaEvent {
    /// The event type identifier.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The index of the output this delta applies to.
    pub output_index: i32,
    /// Unique identifier for the API item associated with this event.
    pub item_id: String,
    /// The incremental input data (delta) for the custom tool call.
    pub delta: String,
}
