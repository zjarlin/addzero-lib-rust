// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionMessageCustomToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageCustomToolCallCustom,
};

/// A call to a custom tool created by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageCustomToolCall {
    /// The ID of the tool call.
    pub id: String,
    /// The type of the tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The custom tool that the model called.
    pub custom: ChatCompletionMessageCustomToolCallCustom,
}
