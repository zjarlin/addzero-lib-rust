// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionResponseChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionResponseMessage,
    CreateChatCompletionResponseChoiceLogprobs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionResponseChoice {
    /// The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop
    /// point or a provided stop sequence, `length` if the maximum number of tokens specified in the request
    /// was reached, `content_filter` if content was omitted due to a flag from our content filters,
    /// `tool_calls` if the model called a tool, or `function_call` (deprecated) if the model called a
    /// function.
    pub finish_reason: String,
    /// The index of the choice in the list of choices.
    pub index: i32,
    pub message: ChatCompletionResponseMessage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CreateChatCompletionResponseChoiceLogprobs>,
}
