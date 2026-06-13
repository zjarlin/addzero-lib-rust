// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestUserMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestUserMessageContentPart,
};

/// The contents of the user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestUserMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestUserMessageContentPart>),
}
