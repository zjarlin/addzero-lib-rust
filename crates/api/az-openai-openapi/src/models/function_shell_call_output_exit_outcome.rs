// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellCallOutputExitOutcome` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that the shell commands finished and returned an exit code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputExitOutcome {
    /// The outcome type. Always `exit`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Exit code from the shell process.
    pub exit_code: i32,
}
