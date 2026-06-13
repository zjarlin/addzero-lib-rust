// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeMCPApprovalResponse` DTO.

use serde::{Deserialize, Serialize};

/// A Realtime item responding to an MCP approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPApprovalResponse {
    /// The type of the item. Always `mcp_approval_response`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the approval response.
    pub id: String,
    /// The ID of the approval request being answered.
    pub approval_request_id: String,
    /// Whether the request was approved.
    pub approve: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
