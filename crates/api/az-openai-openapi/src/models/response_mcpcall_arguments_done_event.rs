// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseMCPCallArgumentsDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the arguments for an MCP tool call are finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPCallArgumentsDoneEvent {
    /// The type of the event. Always 'response.mcp_call_arguments.done'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
    /// A JSON string containing the finalized arguments for the MCP tool call.
    pub arguments: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
