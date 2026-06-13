// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsFunctionObjectFunction` DTO.

use serde::{Deserialize, Serialize};

/// The definition of the function that was called.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFunctionObjectFunction {
    /// The name of the function.
    pub name: String,
    /// The arguments passed to the function.
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
