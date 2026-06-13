// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseUsageInputTokensDetails` DTO.

use serde::{Deserialize, Serialize};

/// A detailed breakdown of the input tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUsageInputTokensDetails {
    /// The number of tokens that were retrieved from the cache. [More on prompt
    /// caching](/docs/guides/prompt-caching).
    pub cached_tokens: i32,
}
