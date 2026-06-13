// Generated from OpenAPI spec. Do not edit by hand.
//! `LocalShellExecAction` DTO.

use serde::{Deserialize, Serialize};

/// Execute a shell command on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalShellExecAction {
    /// The type of the local shell action. Always `exec`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The command to run.
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Environment variables to set for the command.
    pub env: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
