// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsObjectToolCall,
};

/// Details of the tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsObject {
    /// Always `tool_calls`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// An array of tool calls the run step was involved in. These can be associated with one of three types
    /// of tools: `code_interpreter`, `file_search`, or `function`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<RunStepDeltaStepDetailsToolCallsObjectToolCall>>,
}
