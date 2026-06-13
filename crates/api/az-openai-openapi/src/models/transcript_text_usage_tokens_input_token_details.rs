// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptTextUsageTokensInputTokenDetails` DTO.

use serde::{Deserialize, Serialize};

/// Details about the input tokens billed for this request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextUsageTokensInputTokenDetails {
    /// Number of text tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i32>,
    /// Number of audio tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,
}
