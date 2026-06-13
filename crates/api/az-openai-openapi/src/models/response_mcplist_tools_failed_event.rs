// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseMCPListToolsFailedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the attempt to list available MCP tools has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMCPListToolsFailedEvent {
    /// The type of the event. Always 'response.mcp_list_tools.failed'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP tool call item that failed.
    pub item_id: String,
    /// The index of the output item that failed.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
