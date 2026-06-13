// Generated from OpenAPI spec. Do not edit by hand.
//! `CompletionUsagePromptTokensDetails` DTO.

use serde::{Deserialize, Serialize};

/// Breakdown of tokens used in the prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsagePromptTokensDetails {
    /// Audio input tokens present in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,
    /// Cached tokens present in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<i32>,
}
