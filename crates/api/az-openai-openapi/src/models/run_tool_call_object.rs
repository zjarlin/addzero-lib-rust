// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunToolCallObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunToolCallObjectFunction,
};

/// Tool call objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunToolCallObject {
    /// The ID of the tool call. This ID must be referenced when you submit the tool outputs in using the
    /// [Submit tool outputs to run](/docs/api-reference/runs/submitToolOutputs) endpoint.
    pub id: String,
    /// The type of tool call the output is required for. For now, this is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The function definition.
    pub function: RunToolCallObjectFunction,
}
