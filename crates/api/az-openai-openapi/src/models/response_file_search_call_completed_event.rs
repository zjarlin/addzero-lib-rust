// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseFileSearchCallCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a file search call is completed (results found).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFileSearchCallCompletedEvent {
    /// The type of the event. Always `response.file_search_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that the file search call is initiated.
    pub output_index: i32,
    /// The ID of the output item that the file search call is initiated.
    pub item_id: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
