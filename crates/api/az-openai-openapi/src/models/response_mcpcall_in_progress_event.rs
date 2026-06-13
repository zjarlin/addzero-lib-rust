// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPCallInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an MCP tool call is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPCallInProgressEvent {
    /// The type of the event. Always 'response.mcp_call.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The unique identifier of the MCP tool call item being processed.
    pub item_id: String,
}
