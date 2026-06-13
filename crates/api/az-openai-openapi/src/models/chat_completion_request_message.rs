// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestDeveloperMessage,
    ChatCompletionRequestFunctionMessage,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionRequestMessage {
    ChatCompletionRequestDeveloperMessage(ChatCompletionRequestDeveloperMessage),
    ChatCompletionRequestSystemMessage(ChatCompletionRequestSystemMessage),
    ChatCompletionRequestUserMessage(ChatCompletionRequestUserMessage),
    ChatCompletionRequestAssistantMessage(ChatCompletionRequestAssistantMessage),
    ChatCompletionRequestToolMessage(ChatCompletionRequestToolMessage),
    ChatCompletionRequestFunctionMessage(ChatCompletionRequestFunctionMessage),
}
