// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateTranscriptionResponseJsonLogprob` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranscriptionResponseJsonLogprob {
    /// The token in the transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The log probability of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprob: Option<f64>,
    /// The bytes of the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<f64>>,
}
