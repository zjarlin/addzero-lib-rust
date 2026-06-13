// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsCodeOutputImageObject,
    RunStepDetailsToolCallsCodeOutputLogsObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepDetailsToolCallsCodeObjectCodeInterpreterOutput {
    RunStepDetailsToolCallsCodeOutputLogsObject(RunStepDetailsToolCallsCodeOutputLogsObject),
    RunStepDetailsToolCallsCodeOutputImageObject(RunStepDetailsToolCallsCodeOutputImageObject),
}
