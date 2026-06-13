// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateCompletionResponseChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateCompletionResponseChoiceLogprobs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompletionResponseChoice {
    /// The reason the model stopped generating tokens. This will be `stop` if the model hit a natural stop
    /// point or a provided stop sequence, `length` if the maximum number of tokens specified in the request
    /// was reached, or `content_filter` if content was omitted due to a flag from our content filters.
    pub finish_reason: String,
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CreateCompletionResponseChoiceLogprobs>,
    pub text: String,
}
