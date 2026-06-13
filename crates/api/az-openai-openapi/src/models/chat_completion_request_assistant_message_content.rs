// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestAssistantMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestAssistantMessageContentPart,
};

/// The contents of the assistant message. Required unless `tool_calls` or `function_call` is specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestAssistantMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestAssistantMessageContentPart>),
}
