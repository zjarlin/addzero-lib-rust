// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPCallArgumentsDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when there is a delta (partial update) to the arguments of an MCP tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPCallArgumentsDeltaEvent {
    /// The type of the event. Always 'response.mcp_call_arguments.delta'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
    /// A JSON string containing the partial update to the arguments for the MCP tool call.
    pub delta: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
