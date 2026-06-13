// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TranscriptTextDoneEventLogprob` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextDoneEventLogprob {
    /// The token that was used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The log probability of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
    /// The bytes that were used to generate the log probability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<i32>>,
}
