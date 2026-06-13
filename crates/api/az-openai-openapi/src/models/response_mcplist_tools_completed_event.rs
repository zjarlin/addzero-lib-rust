// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPListToolsCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the list of available MCP tools has been successfully retrieved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPListToolsCompletedEvent {
    /// The type of the event. Always 'response.mcp_list_tools.completed'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP tool call item that produced this output.
    pub item_id: String,
    /// The index of the output item that was processed.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
