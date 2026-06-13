// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventMCPListToolsInProgress` DTO.

use serde::{Deserialize, Serialize};

/// Returned when listing MCP tools is in progress for an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventMCPListToolsInProgress {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `mcp_list_tools.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP list tools item.
    pub item_id: String,
}
