// Generated from OpenAPI spec. Do not edit by hand.
//! `ToolChoiceFunction` DTO.

use serde::{Deserialize, Serialize};

/// Use this option to force the model to call a specific function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    /// For function calling, the type is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the function to call.
    pub name: String,
}
