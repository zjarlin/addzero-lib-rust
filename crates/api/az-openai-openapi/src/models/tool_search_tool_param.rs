// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolSearchToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EmptyModelParam,
    ToolSearchExecutionType,
};

/// Hosted or BYOT tool search configuration for deferred tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchToolParam {
    /// The type of the tool. Always `tool_search`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Whether tool search is executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ToolSearchExecutionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<EmptyModelParam>,
}
