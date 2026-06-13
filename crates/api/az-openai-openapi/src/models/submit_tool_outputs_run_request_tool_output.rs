// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `SubmitToolOutputsRunRequestToolOutput` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitToolOutputsRunRequestToolOutput {
    /// The ID of the tool call in the `required_action` object within the run object the output is being
    /// submitted for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// The output of the tool call to be submitted to continue the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}
