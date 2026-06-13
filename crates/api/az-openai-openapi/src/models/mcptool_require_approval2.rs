// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MCPToolRequireApproval2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPToolRequireApproval2MCPToolApprovalFilter,
};

/// Specify which of the MCP server's tools require approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPToolRequireApproval2 {
    MCPToolApprovalFilter(MCPToolRequireApproval2MCPToolApprovalFilter),
    MCPToolApprovalSetting(String),
}
