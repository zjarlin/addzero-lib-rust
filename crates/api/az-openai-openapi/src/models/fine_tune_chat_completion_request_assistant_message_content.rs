// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneChatCompletionRequestAssistantMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestAssistantMessageContentPart,
};

/// The contents of the assistant message. Required unless `tool_calls` or `function_call` is specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneChatCompletionRequestAssistantMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestAssistantMessageContentPart>),
}
