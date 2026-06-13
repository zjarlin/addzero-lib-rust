// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeMCPHTTPError` DTO.

use serde::{Deserialize, Serialize};

/// Realtime MCP HTTP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMCPHTTPError {
    #[serde(rename = "type")]
    pub type_value: String,
    pub code: i32,
    pub message: String,
}
