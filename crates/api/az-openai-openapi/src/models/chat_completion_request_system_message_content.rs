// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestSystemMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestSystemMessageContentPart,
};

/// The contents of the system message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestSystemMessageContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestSystemMessageContentPart>),
}
