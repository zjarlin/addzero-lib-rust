// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsFunctionObjectFunction` DTO.

use serde::{Deserialize, Serialize};

/// The definition of the function that was called.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFunctionObjectFunction {
    /// The name of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The arguments passed to the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
