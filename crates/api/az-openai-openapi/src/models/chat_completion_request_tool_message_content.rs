// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestToolMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestToolMessageContentPart,
};

/// The contents of the tool message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestToolMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestToolMessageContentPart>),
}
