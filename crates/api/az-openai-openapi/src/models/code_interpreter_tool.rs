// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CodeInterpreterTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CodeInterpreterToolContainer,
};

/// A tool that runs Python code to help generate a response to a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterTool {
    /// The type of the code interpreter tool. Always `code_interpreter`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The code interpreter container. Can be a container ID or an object that specifies uploaded file IDs
    /// to make available to your code, along with an optional `memory_limit` setting.
    pub container: CodeInterpreterToolContainer,
}
