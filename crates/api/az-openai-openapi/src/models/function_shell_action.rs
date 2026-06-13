// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellAction` DTO.

use serde::{Deserialize, Serialize};

/// Execute a shell command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellAction {
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<i32>,
}
