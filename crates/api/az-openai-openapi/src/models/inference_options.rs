// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InferenceOptions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ToolChoice,
};

/// Model and tool overrides applied when generating the assistant response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
