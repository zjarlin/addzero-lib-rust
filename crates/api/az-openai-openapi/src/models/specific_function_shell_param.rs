// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `SpecificFunctionShellParam` DTO.

use serde::{Deserialize, Serialize};

/// Forces the model to call the shell tool when a tool call is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificFunctionShellParam {
    /// The tool to call. Always `shell`.
    #[serde(rename = "type")]
    pub type_value: String,
}
