// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionTokenLogprob` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionTokenLogprobTopLogprob,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionTokenLogprob {
    /// The token.
    pub token: String,
    /// The log probability of this token, if it is within the top 20 most likely tokens. Otherwise, the
    /// value `-9999.0` is used to signify that the token is very unlikely.
    pub logprob: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i32>>,
    /// List of the most likely tokens and their log probability, at this token position. The number of
    /// entries may be fewer than the requested `top_logprobs`.
    pub top_logprobs: Vec<ChatCompletionTokenLogprobTopLogprob>,
}
