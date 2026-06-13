// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeMCPProtocolError` DTO.

use serde::{Deserialize, Serialize};

/// Realtime MCP protocol error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPProtocolError {
    #[serde(rename = "type")]
    pub type_value: String,
    pub code: i32,
    pub message: String,
}
