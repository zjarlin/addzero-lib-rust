// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseMCPListToolsInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the system is in the process of retrieving the list of available MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPListToolsInProgressEvent {
    /// The type of the event. Always 'response.mcp_list_tools.in_progress'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP tool call item that is being processed.
    pub item_id: String,
    /// The index of the output item that is being processed.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
