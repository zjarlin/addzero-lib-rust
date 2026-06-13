// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunObjectRequiredActionSubmitToolOutputs` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunToolCallObject,
};

/// Details on the tool outputs needed for this run to continue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObjectRequiredActionSubmitToolOutputs {
    /// A list of the relevant tool calls.
    pub tool_calls: Vec<RunToolCallObject>,
}
