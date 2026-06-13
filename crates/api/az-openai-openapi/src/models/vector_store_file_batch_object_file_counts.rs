// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreFileBatchObjectFileCounts` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileBatchObjectFileCounts {
    /// The number of files that are currently being processed.
    pub in_progress: i32,
    /// The number of files that have been processed.
    pub completed: i32,
    /// The number of files that have failed to process.
    pub failed: i32,
    /// The number of files that where cancelled.
    pub cancelled: i32,
    /// The total number of files.
    pub total: i32,
}
