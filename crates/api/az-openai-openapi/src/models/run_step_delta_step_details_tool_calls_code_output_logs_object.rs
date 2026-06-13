// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject` DTO.

use serde::{Deserialize, Serialize};

/// Text output from the Code Interpreter tool call as part of a run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject {
    /// The index of the output in the outputs array.
    pub index: i32,
    /// Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text output from the Code Interpreter tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs: Option<String>,
}
