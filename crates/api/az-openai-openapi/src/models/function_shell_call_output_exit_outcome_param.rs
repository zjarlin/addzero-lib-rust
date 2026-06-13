// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallOutputExitOutcomeParam` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that the shell commands finished and returned an exit code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputExitOutcomeParam {
    /// The outcome type. Always `exit`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The exit code returned by the shell process.
    pub exit_code: i32,
}
