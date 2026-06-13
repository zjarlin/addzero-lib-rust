// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunCompletionUsage2` DTO.

use serde::{Deserialize, Serialize};

/// Usage statistics related to the run. This value will be `null` if the run is not in a terminal state
/// (i.e. `in_progress`, `queued`, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCompletionUsage2 {
    /// Number of completion tokens used over the course of the run.
    pub completion_tokens: i32,
    /// Number of prompt tokens used over the course of the run.
    pub prompt_tokens: i32,
    /// Total number of tokens used (prompt + completion).
    pub total_tokens: i32,
}
