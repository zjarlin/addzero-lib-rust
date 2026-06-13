// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPCallCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an MCP tool call has completed successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPCallCompletedEvent {
    /// The type of the event. Always 'response.mcp_call.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP tool call item that completed.
    pub item_id: String,
    /// The index of the output item that completed.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
