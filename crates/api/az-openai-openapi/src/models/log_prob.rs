// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `LogProb` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TopLogProb,
};

/// The log probability of a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<i32>,
    pub top_logprobs: Vec<TopLogProb>,
}
