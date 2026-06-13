// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolSearchCall` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    FunctionCallStatus,
    ToolSearchExecutionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchCall {
    /// The type of the item. Always `tool_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the tool search call item.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// Whether tool search was executed by the server or by the client.
    pub execution: ToolSearchExecutionType,
    /// Arguments used for the tool search call.
    pub arguments: OpenAiJsonValue,
    /// The status of the tool search call item that was recorded.
    pub status: FunctionCallStatus,
    /// The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}
