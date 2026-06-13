// Generated from OpenAPI spec. Do not edit by hand.
//! `TopLogProb` DTO.

use serde::{Deserialize, Serialize};

/// The top log probability of a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<i32>,
}
