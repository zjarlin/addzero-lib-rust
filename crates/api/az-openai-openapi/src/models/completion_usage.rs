// Generated from OpenAPI spec. Do not edit by hand.
//! `CompletionUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CompletionUsageCompletionTokensDetails,
    CompletionUsagePromptTokensDetails,
};

/// Usage statistics for the completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionUsage {
    /// Number of tokens in the generated completion.
    pub completion_tokens: i32,
    /// Number of tokens in the prompt.
    pub prompt_tokens: i32,
    /// Total number of tokens used in the request (prompt + completion).
    pub total_tokens: i32,
    /// Breakdown of tokens used in a completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionUsageCompletionTokensDetails>,
    /// Breakdown of tokens used in the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<CompletionUsagePromptTokensDetails>,
}
