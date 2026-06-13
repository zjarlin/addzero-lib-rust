// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestDeveloperMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartText,
};

/// The contents of the developer message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestDeveloperMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestMessageContentPartText>),
}
