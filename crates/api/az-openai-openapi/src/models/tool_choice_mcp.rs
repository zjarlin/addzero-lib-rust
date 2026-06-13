// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoiceMCP` DTO.

use serde::{Deserialize, Serialize};

/// Use this option to force the model to call a specific tool on a remote MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceMCP {
    /// For MCP tools, the type is always `mcp`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The label of the MCP server to use.
    pub server_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
