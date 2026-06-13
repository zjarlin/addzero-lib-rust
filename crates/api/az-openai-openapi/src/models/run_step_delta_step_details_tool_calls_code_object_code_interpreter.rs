// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput,
};

/// The Code Interpreter tool call definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreter {
    /// The input to the Code Interpreter tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    /// The outputs from the Code Interpreter tool call. Code Interpreter can output one or more items,
    /// including text (`logs`) or images (`image`). Each of these are represented by a different object
    /// type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput>>,
}
