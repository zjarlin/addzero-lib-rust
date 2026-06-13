// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunPerModelUsageItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunPerModelUsageItem {
    /// The name of the model.
    pub model_name: String,
    /// The number of invocations.
    pub invocation_count: i32,
    /// The number of prompt tokens used.
    pub prompt_tokens: i32,
    /// The number of completion tokens generated.
    pub completion_tokens: i32,
    /// The total number of tokens used.
    pub total_tokens: i32,
    /// The number of tokens retrieved from cache.
    pub cached_tokens: i32,
}
