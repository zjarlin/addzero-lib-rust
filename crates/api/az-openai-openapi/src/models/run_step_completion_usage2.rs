// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepCompletionUsage2` DTO.

use serde::{Deserialize, Serialize};

/// Usage statistics related to the run step. This value will be `null` while the run step's status is
/// `in_progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepCompletionUsage2 {
    /// Number of completion tokens used over the course of the run step.
    pub completion_tokens: i32,
    /// Number of prompt tokens used over the course of the run step.
    pub prompt_tokens: i32,
    /// Total number of tokens used (prompt + completion).
    pub total_tokens: i32,
}
