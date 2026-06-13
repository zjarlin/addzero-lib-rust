// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallOutputContentParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionShellCallOutputOutcomeParam,
};

/// Captured stdout and stderr for a portion of a shell tool call output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputContentParam {
    /// Captured stdout output for the shell call.
    pub stdout: String,
    /// Captured stderr output for the shell call.
    pub stderr: String,
    /// The exit or timeout outcome associated with this shell call.
    pub outcome: FunctionShellCallOutputOutcomeParam,
}
