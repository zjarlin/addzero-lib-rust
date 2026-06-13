// Generated from OpenAPI spec. Do not edit by hand.
//! `LogProbProperties` DTO.

use serde::{Deserialize, Serialize};

/// A log probability object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProbProperties {
    /// The token that was used to generate the log probability.
    pub token: String,
    /// The log probability of the token.
    pub logprob: f64,
    /// The bytes that were used to generate the log probability.
    pub bytes: Vec<i32>,
}
