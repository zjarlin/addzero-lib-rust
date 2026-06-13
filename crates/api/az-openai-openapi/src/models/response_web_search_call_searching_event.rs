// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseWebSearchCallSearchingEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a web search call is executing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWebSearchCallSearchingEvent {
    /// The type of the event. Always `response.web_search_call.searching`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that the web search call is associated with.
    pub output_index: i32,
    /// Unique ID for the output item associated with the web search call.
    pub item_id: String,
    /// The sequence number of the web search call being processed.
    pub sequence_number: i32,
}
