// Generated from OpenAPI spec. Do not edit by hand.
//! `CodeInterpreterTextOutput` DTO.

use serde::{Deserialize, Serialize};

/// The output of a code interpreter tool call that is text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterTextOutput {
    /// The type of the code interpreter text output. Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The logs of the code interpreter tool call.
    pub logs: String,
}
