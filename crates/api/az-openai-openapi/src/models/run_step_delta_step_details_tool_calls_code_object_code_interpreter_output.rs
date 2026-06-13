// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsCodeOutputImageObject,
    RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaStepDetailsToolCallsCodeObjectCodeInterpreterOutput {
    RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject(RunStepDeltaStepDetailsToolCallsCodeOutputLogsObject),
    RunStepDeltaStepDetailsToolCallsCodeOutputImageObject(RunStepDeltaStepDetailsToolCallsCodeOutputImageObject),
}
