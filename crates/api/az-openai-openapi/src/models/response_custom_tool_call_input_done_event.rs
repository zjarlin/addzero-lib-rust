// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseCustomToolCallInputDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Event indicating that input for a custom tool call is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCustomToolCallInputDoneEvent {
    /// The event type identifier.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The index of the output this event applies to.
    pub output_index: i32,
    /// Unique identifier for the API item associated with this event.
    pub item_id: String,
    /// The complete input data for the custom tool call.
    pub input: String,
}
