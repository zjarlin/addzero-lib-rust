// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionAllowedToolsChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionAllowedTools,
};

/// Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionAllowedToolsChoice {
    /// Allowed tool configuration type. Always `allowed_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub allowed_tools: ChatCompletionAllowedTools,
}
