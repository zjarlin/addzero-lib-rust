// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseProperties` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ModelIdsResponses,
    Prompt,
    Reasoning,
    ResponseTextParam,
    ToolChoiceParam,
    ToolsArray,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Model ID used to generate the response, like `gpt-4o` or `o3`. OpenAI offers a wide range of models
    /// with different capabilities, performance characteristics, and price points. Refer to the [model
    /// guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelIdsResponses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsArray>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoiceParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Prompt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
}
