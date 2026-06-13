// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalRunResultCounts` DTO.

use serde::{Deserialize, Serialize};

/// Counters summarizing the outcomes of the evaluation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRunResultCounts {
    /// Total number of executed output items.
    pub total: i32,
    /// Number of output items that resulted in an error.
    pub errored: i32,
    /// Number of output items that failed to pass the evaluation.
    pub failed: i32,
    /// Number of output items that passed the evaluation.
    pub passed: i32,
}
