// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeMCPApprovalRequest` DTO.

use serde::{Deserialize, Serialize};

/// A Realtime item requesting human approval of a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPApprovalRequest {
    /// The type of the item. Always `mcp_approval_request`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the approval request.
    pub id: String,
    /// The label of the MCP server making the request.
    pub server_label: String,
    /// The name of the tool to run.
    pub name: String,
    /// A JSON string of arguments for the tool.
    pub arguments: String,
}
