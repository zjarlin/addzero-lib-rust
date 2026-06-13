// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `SubmitToolOutputsRunRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    SubmitToolOutputsRunRequestToolOutput,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitToolOutputsRunRequest {
    /// A list of tools for which the outputs are being submitted.
    pub tool_outputs: Vec<SubmitToolOutputsRunRequestToolOutput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}
