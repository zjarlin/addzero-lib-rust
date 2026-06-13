// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventResponseMCPCallCompleted` DTO.

use serde::{Deserialize, Serialize};

/// Returned when an MCP tool call has completed successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventResponseMCPCallCompleted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.mcp_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The ID of the MCP tool call item.
    pub item_id: String,
}
