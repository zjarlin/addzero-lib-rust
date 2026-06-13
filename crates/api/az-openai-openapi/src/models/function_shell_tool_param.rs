// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionShellToolParamEnvironment,
};

/// A tool that allows the model to execute shell commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionShellToolParam {
    /// The type of the shell tool. Always `shell`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<FunctionShellToolParamEnvironment>,
}
