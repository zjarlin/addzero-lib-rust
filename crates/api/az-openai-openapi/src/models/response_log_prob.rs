// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseLogProb` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseLogProbTopLogprob,
};

/// A logprob is the logarithmic probability that the model assigns to producing a particular token at a
/// given position in the sequence. Less-negative (higher) logprob values indicate greater model
/// confidence in that token choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLogProb {
    /// A possible text token.
    pub token: String,
    /// The log probability of this token.
    pub logprob: f64,
    /// The log probabilities of up to 20 of the most likely tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<ResponseLogProbTopLogprob>>,
}
