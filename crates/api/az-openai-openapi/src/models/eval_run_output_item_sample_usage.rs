// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunOutputItemSampleUsage` DTO.

use serde::{Deserialize, Serialize};

/// Token usage details for the sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunOutputItemSampleUsage {
    /// The total number of tokens used.
    pub total_tokens: i32,
    /// The number of completion tokens generated.
    pub completion_tokens: i32,
    /// The number of prompt tokens used.
    pub prompt_tokens: i32,
    /// The number of tokens retrieved from cache.
    pub cached_tokens: i32,
}
