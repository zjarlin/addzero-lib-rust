// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsCodeOutputLogsObject` DTO.

use serde::{Deserialize, Serialize};

/// Text output from the Code Interpreter tool call as part of a run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsCodeOutputLogsObject {
    /// Always `logs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text output from the Code Interpreter tool call.
    pub logs: String,
}
