// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPCallFailedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an MCP tool call has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPCallFailedEvent {
    /// The type of the event. Always 'response.mcp_call.failed'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP tool call item that failed.
    pub item_id: String,
    /// The index of the output item that failed.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
