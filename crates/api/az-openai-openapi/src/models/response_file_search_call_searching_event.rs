// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFileSearchCallSearchingEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a file search is currently searching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFileSearchCallSearchingEvent {
    /// The type of the event. Always `response.file_search_call.searching`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that the file search call is searching.
    pub output_index: i32,
    /// The ID of the output item that the file search call is initiated.
    pub item_id: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
