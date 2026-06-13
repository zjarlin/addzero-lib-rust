// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsToolCallsCodeObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsCodeObjectCodeInterpreter,
};

/// Details of the Code Interpreter tool call the run step was involved in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsCodeObject {
    /// The ID of the tool call.
    pub id: String,
    /// The type of tool call. This is always going to be `code_interpreter` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The Code Interpreter tool call definition.
    pub code_interpreter: RunStepDetailsToolCallsCodeObjectCodeInterpreter,
}
