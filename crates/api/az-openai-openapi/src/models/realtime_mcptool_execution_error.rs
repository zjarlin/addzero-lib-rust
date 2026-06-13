// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeMCPToolExecutionError` DTO.

use serde::{Deserialize, Serialize};

/// Realtime MCP tool execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPToolExecutionError {
    #[serde(rename = "type")]
    pub type_value: String,
    pub message: String,
}
