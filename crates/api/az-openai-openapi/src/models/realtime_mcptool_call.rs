// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeMCPToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeMCPToolCallError,
};

/// A Realtime item representing an invocation of a tool on an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPToolCall {
    /// The type of the item. Always `mcp_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the tool call.
    pub id: String,
    /// The label of the MCP server running the tool.
    pub server_label: String,
    /// The name of the tool that was run.
    pub name: String,
    /// A JSON string of the arguments passed to the tool.
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RealtimeMCPToolCallError>,
}
