// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionMessageToolCallChunk` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageToolCallChunkFunction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageToolCallChunk {
    pub index: i32,
    /// The ID of the tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<ChatCompletionMessageToolCallChunkFunction>,
}
