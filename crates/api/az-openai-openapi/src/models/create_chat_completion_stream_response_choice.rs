// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionStreamResponseChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionStreamResponseDelta,
    CreateChatCompletionStreamResponseChoiceLogprobs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionStreamResponseChoice {
    pub delta: ChatCompletionStreamResponseDelta,
    /// Log probability information for the choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CreateChatCompletionStreamResponseChoiceLogprobs>,
    /// The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop
    /// point or a provided stop sequence, `length` if the maximum number of tokens specified in the request
    /// was reached, `content_filter` if content was omitted due to a flag from our content filters,
    /// `tool_calls` if the model called a tool, or `function_call` (deprecated) if the model called a
    /// function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    /// The index of the choice in the list of choices.
    pub index: i32,
}
