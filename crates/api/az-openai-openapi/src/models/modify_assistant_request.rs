// Generated from OpenAPI spec. Do not edit by hand.
//! `ModifyAssistantRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantsApiResponseFormatOption,
    Metadata,
    ModifyAssistantRequestModel,
    ModifyAssistantRequestTool,
    ModifyAssistantRequestToolResources,
    ReasoningEffort,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyAssistantRequest {
    /// ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see
    /// all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModifyAssistantRequestModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// A list of tool enabled on the assistant. There can be a maximum of 128 tools per assistant. Tools
    /// can be of types `code_interpreter`, `file_search`, or `function`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ModifyAssistantRequestTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<ModifyAssistantRequestToolResources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantsApiResponseFormatOption>,
}
