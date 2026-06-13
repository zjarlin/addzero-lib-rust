// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsFunctionObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsFunctionObjectFunction,
};

/// Function tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFunctionObject {
    /// The index of the tool call in the tool calls array.
    pub index: i32,
    /// The ID of the tool call object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of tool call. This is always going to be `function` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The definition of the function that was called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<RunStepDeltaStepDetailsToolCallsFunctionObjectFunction>,
}
