// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseLogProbTopLogprob` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLogProbTopLogprob {
    /// A possible text token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The log probability of this token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
}
