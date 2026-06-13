// Generated from OpenAPI spec. Do not edit by hand.
//! `CodeInterpreterToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CodeInterpreterToolCallOutput,
};

/// A tool call to run code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterToolCall {
    /// The type of the code interpreter tool call. Always `code_interpreter_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the code interpreter tool call.
    pub id: String,
    /// The status of the code interpreter tool call. Valid values are `in_progress`, `completed`,
    /// `incomplete`, `interpreting`, and `failed`.
    pub status: String,
    /// The ID of the container used to run the code.
    pub container_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<CodeInterpreterToolCallOutput>>,
}
