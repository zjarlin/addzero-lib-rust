// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolSearchOutputItemParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionCallItemStatus,
    Tool,
    ToolSearchExecutionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchOutputItemParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// The item type. Always `tool_search_output`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Whether tool search was executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ToolSearchExecutionType>,
    /// The loaded tool definitions returned by the tool search output.
    pub tools: Vec<Tool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<FunctionCallItemStatus>,
}
