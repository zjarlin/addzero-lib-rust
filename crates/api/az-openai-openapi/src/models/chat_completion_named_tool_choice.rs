// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionNamedToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionNamedToolChoiceFunction,
};

/// Specifies a tool the model should use. Use to force the model to call a specific function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedToolChoice {
    /// For function calling, the type is always `function`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub function: ChatCompletionNamedToolChoiceFunction,
}
