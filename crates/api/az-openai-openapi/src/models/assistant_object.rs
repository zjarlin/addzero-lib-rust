// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AssistantObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AssistantObjectTool,
    AssistantObjectToolResources,
    AssistantsApiResponseFormatOption,
    Metadata,
};

/// Represents an `assistant` that can call the model and use tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `assistant`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the assistant was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ID of the model to use. You can use the [List models](/docs/api-reference/models/list) API to see
    /// all of your available models, or see our [Model overview](/docs/models) for descriptions of them.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// A list of tool enabled on the assistant. There can be a maximum of 128 tools per assistant. Tools
    /// can be of types `code_interpreter`, `file_search`, or `function`.
    pub tools: Vec<AssistantObjectTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<AssistantObjectToolResources>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<AssistantsApiResponseFormatOption>,
}
