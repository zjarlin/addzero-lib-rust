// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventMCPListToolsCompleted` DTO.

use serde::{Deserialize, Serialize};

/// Returned when listing MCP tools has completed for an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventMCPListToolsCompleted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `mcp_list_tools.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the MCP list tools item.
    pub item_id: String,
}
