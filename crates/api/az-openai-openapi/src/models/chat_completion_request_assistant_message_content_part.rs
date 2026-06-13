// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestAssistantMessageContentPart` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartRefusal,
    ChatCompletionRequestMessageContentPartText,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestAssistantMessageContentPart {
    ChatCompletionRequestMessageContentPartText(ChatCompletionRequestMessageContentPartText),
    ChatCompletionRequestMessageContentPartRefusal(ChatCompletionRequestMessageContentPartRefusal),
}
