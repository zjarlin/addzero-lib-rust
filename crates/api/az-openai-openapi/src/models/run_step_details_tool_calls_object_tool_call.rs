// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsToolCallsObjectToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsCodeObject,
    RunStepDetailsToolCallsFileSearchObject,
    RunStepDetailsToolCallsFunctionObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepDetailsToolCallsObjectToolCall {
    RunStepDetailsToolCallsCodeObject(RunStepDetailsToolCallsCodeObject),
    RunStepDetailsToolCallsFileSearchObject(RunStepDetailsToolCallsFileSearchObject),
    RunStepDetailsToolCallsFunctionObject(RunStepDetailsToolCallsFunctionObject),
}
