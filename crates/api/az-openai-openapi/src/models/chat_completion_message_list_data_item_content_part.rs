// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionMessageListDataItemContentPart` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestMessageContentPartText,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionMessageListDataItemContentPart {
    ChatCompletionRequestMessageContentPartText(ChatCompletionRequestMessageContentPartText),
    ChatCompletionRequestMessageContentPartImage(ChatCompletionRequestMessageContentPartImage),
}
