// Generated from OpenAPI spec. Do not edit by hand.
//! `MCPToolFilter` DTO.

use serde::{Deserialize, Serialize};

/// A filter object to specify which tools are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolFilter {
    /// List of allowed tool names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_names: Option<Vec<String>>,
    /// Indicates whether or not a tool modifies data or is read-only. If an MCP server is [annotated with
    /// `readOnlyHint`](https://modelcontextprotocol.io/specification/2025-06-18/schema#toolannotations-
    /// readonlyhint), it will match this filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}
