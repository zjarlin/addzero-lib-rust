// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsToolCallsFunctionObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsFunctionObjectFunction,
};

/// Function tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFunctionObject {
    /// The ID of the tool call object.
    pub id: String,
    /// The type of tool call. This is always going to be `function` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The definition of the function that was called.
    pub function: RunStepDetailsToolCallsFunctionObjectFunction,
}
