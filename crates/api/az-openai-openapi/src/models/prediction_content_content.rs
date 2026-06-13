// Generated from OpenAPI spec. Do not edit by hand.
//! `PredictionContentContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartText,
};

/// The content that should be matched when generating a model response. If generated tokens would match
/// this content, the entire model response can be returned much more quickly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PredictionContentContent {
    TextContent(String),
    ArrayOfContentParts(Vec<ChatCompletionRequestMessageContentPartText>),
}
