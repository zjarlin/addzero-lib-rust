// Generated from OpenAPI spec. Do not edit by hand.
//! `CodeInterpreterOutputLogs` DTO.

use serde::{Deserialize, Serialize};

/// The logs output from the code interpreter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterOutputLogs {
    /// The type of the output. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The logs output from the code interpreter.
    pub logs: String,
}
