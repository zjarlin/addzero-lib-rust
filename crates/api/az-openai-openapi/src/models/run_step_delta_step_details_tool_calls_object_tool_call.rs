// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsObjectToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsCodeObject,
    RunStepDeltaStepDetailsToolCallsFileSearchObject,
    RunStepDeltaStepDetailsToolCallsFunctionObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepDeltaStepDetailsToolCallsObjectToolCall {
    RunStepDeltaStepDetailsToolCallsCodeObject(RunStepDeltaStepDetailsToolCallsCodeObject),
    RunStepDeltaStepDetailsToolCallsFileSearchObject(RunStepDeltaStepDetailsToolCallsFileSearchObject),
    RunStepDeltaStepDetailsToolCallsFunctionObject(RunStepDeltaStepDetailsToolCallsFunctionObject),
}
