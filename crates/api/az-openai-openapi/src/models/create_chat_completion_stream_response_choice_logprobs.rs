// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionStreamResponseChoiceLogprobs` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionTokenLogprob,
};

/// Log probability information for the choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionStreamResponseChoiceLogprobs {
    /// A list of message content tokens with log probability information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    /// A list of message refusal tokens with log probability information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}
