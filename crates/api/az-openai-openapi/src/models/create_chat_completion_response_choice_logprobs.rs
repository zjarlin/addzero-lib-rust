// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionResponseChoiceLogprobs` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionTokenLogprob,
};

/// Log probability information for the choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionResponseChoiceLogprobs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}
