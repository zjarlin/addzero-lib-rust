// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolSearchCallItemParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EmptyModelParam,
    FunctionCallItemStatus,
    ToolSearchExecutionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchCallItemParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// The item type. Always `tool_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Whether tool search was executed by the server or by the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ToolSearchExecutionType>,
    /// The arguments supplied to the tool search call.
    pub arguments: EmptyModelParam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<FunctionCallItemStatus>,
}
