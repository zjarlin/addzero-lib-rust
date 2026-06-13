// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeMCPListTools` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPListToolsTool,
};

/// A Realtime item listing tools available on an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPListTools {
    /// The type of the item. Always `mcp_list_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The label of the MCP server.
    pub server_label: String,
    /// The tools available on the server.
    pub tools: Vec<MCPListToolsTool>,
}
