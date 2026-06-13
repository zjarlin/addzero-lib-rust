// Generated from OpenAPI spec. Do not edit by hand.
//! `MCPListTools` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPListToolsTool,
};

/// A list of tools available on an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPListTools {
    /// The type of the item. Always `mcp_list_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the list.
    pub id: String,
    /// The label of the MCP server.
    pub server_label: String,
    /// The tools available on the server.
    pub tools: Vec<MCPListToolsTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
