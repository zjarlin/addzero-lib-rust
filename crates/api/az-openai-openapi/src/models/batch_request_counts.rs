// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `BatchRequestCounts` DTO.

use serde::{Deserialize, Serialize};

/// The request counts for different statuses within the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total number of requests in the batch.
    pub total: i32,
    /// Number of requests that have been completed successfully.
    pub completed: i32,
    /// Number of requests that have failed.
    pub failed: i32,
}
