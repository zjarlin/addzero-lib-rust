// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    VectorStoreExpirationAfter,
    VectorStoreObjectFileCounts,
};

/// A vector store is a collection of processed files can be used by the `file_search` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `vector_store`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the vector store was created.
    pub created_at: i64,
    /// The name of the vector store.
    pub name: String,
    /// The total number of bytes used by the files in the vector store.
    pub usage_bytes: i32,
    pub file_counts: VectorStoreObjectFileCounts,
    /// The status of the vector store, which can be either `expired`, `in_progress`, or `completed`. A
    /// status of `completed` indicates that the vector store is ready for use.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<VectorStoreExpirationAfter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
