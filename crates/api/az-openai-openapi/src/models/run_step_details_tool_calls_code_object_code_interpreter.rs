// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsCodeObjectCodeInterpreter` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput,
};

/// The Code Interpreter tool call definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsCodeObjectCodeInterpreter {
    /// The input to the Code Interpreter tool call.
    pub input: String,
    /// The outputs from the Code Interpreter tool call. Code Interpreter can output one or more items,
    /// including text (`logs`) or images (`image`). Each of these are represented by a different object
    /// type.
    pub outputs: Vec<RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput>,
}
