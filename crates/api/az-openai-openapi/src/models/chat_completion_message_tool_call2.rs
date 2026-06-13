// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionMessageToolCall2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageCustomToolCall,
    ChatCompletionMessageToolCall,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionMessageToolCall2 {
    ChatCompletionMessageToolCall(ChatCompletionMessageToolCall),
    ChatCompletionMessageCustomToolCall(ChatCompletionMessageCustomToolCall),
}
