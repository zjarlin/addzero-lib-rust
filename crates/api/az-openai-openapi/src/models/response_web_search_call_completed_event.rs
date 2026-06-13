// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseWebSearchCallCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a web search call is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWebSearchCallCompletedEvent {
    /// The type of the event. Always `response.web_search_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item that the web search call is associated with.
    pub output_index: i32,
    /// Unique ID for the output item associated with the web search call.
    pub item_id: String,
    /// The sequence number of the web search call being processed.
    pub sequence_number: i32,
}
