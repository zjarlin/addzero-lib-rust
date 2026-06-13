// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallOutputContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionShellCallOutputContentOutcome,
};

/// The content of a shell tool call output that was emitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellCallOutputContent {
    /// The standard output that was captured.
    pub stdout: String,
    /// The standard error output that was captured.
    pub stderr: String,
    /// Represents either an exit outcome (with an exit code) or a timeout outcome for a shell call output
    /// chunk.
    pub outcome: FunctionShellCallOutputContentOutcome,
    /// The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}
