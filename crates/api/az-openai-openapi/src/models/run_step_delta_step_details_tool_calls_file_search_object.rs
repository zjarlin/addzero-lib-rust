// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsFileSearchObject` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// File search tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsFileSearchObject {
    /// The index of the tool call in the tool calls array.
    pub index: i32,
    /// The ID of the tool call object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of tool call. This is always going to be `file_search` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// For now, this is always going to be an empty object.
    pub file_search: OpenAiJsonObject,
}
