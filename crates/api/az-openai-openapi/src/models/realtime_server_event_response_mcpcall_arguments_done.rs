// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventResponseMCPCallArgumentsDone` DTO.

use serde::{Deserialize, Serialize};

/// Returned when MCP tool call arguments are finalized during response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventResponseMCPCallArgumentsDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.mcp_call_arguments.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the MCP tool call item.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The final JSON-encoded arguments string.
    pub arguments: String,
}
