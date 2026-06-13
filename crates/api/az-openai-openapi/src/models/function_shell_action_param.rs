// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellActionParam` DTO.

use serde::{Deserialize, Serialize};

/// Commands and limits describing how to run the shell tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellActionParam {
    /// Ordered shell commands for the execution environment to run.
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<i32>,
}
