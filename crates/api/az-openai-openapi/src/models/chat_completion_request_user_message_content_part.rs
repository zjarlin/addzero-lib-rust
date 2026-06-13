// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionRequestUserMessageContentPart` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartAudio,
    ChatCompletionRequestMessageContentPartFile,
    ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestMessageContentPartText,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestUserMessageContentPart {
    ChatCompletionRequestMessageContentPartText(ChatCompletionRequestMessageContentPartText),
    ChatCompletionRequestMessageContentPartImage(ChatCompletionRequestMessageContentPartImage),
    ChatCompletionRequestMessageContentPartAudio(ChatCompletionRequestMessageContentPartAudio),
    ChatCompletionRequestMessageContentPartFile(ChatCompletionRequestMessageContentPartFile),
}
