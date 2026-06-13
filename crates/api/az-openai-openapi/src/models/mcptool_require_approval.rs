// Generated from OpenAPI spec. Do not edit by hand.
//! `MCPToolRequireApproval` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MCPToolFilter,
};

/// Specify which of the MCP server's tools require approval. Can be `always`, `never`, or a filter
/// object associated with tools that require approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolRequireApproval {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always: Option<MCPToolFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub never: Option<MCPToolFilter>,
}
