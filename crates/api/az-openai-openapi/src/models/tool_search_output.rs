// Generated from OpenAPI spec. Do not edit by hand.
//! `ToolSearchOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionCallOutputStatusEnum,
    Tool,
    ToolSearchExecutionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchOutput {
    /// The type of the item. Always `tool_search_output`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the tool search output item.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// Whether tool search was executed by the server or by the client.
    pub execution: ToolSearchExecutionType,
    /// The loaded tool definitions returned by tool search.
    pub tools: Vec<Tool>,
    /// The status of the tool search output item that was recorded.
    pub status: FunctionCallOutputStatusEnum,
    /// The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}
