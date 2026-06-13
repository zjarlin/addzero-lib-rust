// Generated from OpenAPI spec. Do not edit by hand.
//! `LocalShellToolParam` DTO.

use serde::{Deserialize, Serialize};

/// A tool that allows the model to execute shell commands in a local environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalShellToolParam {
    /// The type of the local shell tool. Always `local_shell`.
    #[serde(rename = "type")]
    pub type_value: String,
}
