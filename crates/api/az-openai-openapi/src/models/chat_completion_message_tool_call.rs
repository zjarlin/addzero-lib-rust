// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionMessageToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageToolCallFunction,
};

/// A call to a function tool created by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageToolCall {
    /// The ID of the tool call.
    pub id: String,
    /// The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The function that the model called.
    pub function: ChatCompletionMessageToolCallFunction,
}
