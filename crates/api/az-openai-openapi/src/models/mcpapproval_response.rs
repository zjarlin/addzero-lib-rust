// Generated from OpenAPI spec. Do not edit by hand.
//! `MCPApprovalResponse` DTO.

use serde::{Deserialize, Serialize};

/// A response to an MCP approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPApprovalResponse {
    /// The type of the item. Always `mcp_approval_response`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ID of the approval request being answered.
    pub approval_request_id: String,
    /// Whether the request was approved.
    pub approve: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
