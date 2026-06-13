// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter,
};

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeObject {
    /// The index of the tool call in the tool calls array.
    pub index: i32,
    /// The ID of the tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of tool call. This is always going to be `code_interpreter` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The Code Interpreter tool call definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_interpreter: Option<RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter>,
}
