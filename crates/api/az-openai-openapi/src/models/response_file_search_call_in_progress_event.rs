// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFileSearchCallInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a file search call is initiated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFileSearchCallInProgressEvent {
    /// The type of the event. Always `response.file_search_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that the file search call is initiated.
    pub output_index: i32,
    /// The ID of the output item that the file search call is initiated.
    pub item_id: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
