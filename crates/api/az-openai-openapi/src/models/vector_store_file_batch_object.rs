// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreFileBatchObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileBatchObjectFileCounts,
};

/// A batch of files attached to a vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileBatchObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `vector_store.file_batch`.
    pub object: String,
    /// The Unix timestamp (in seconds) for when the vector store files batch was created.
    pub created_at: i64,
    /// The ID of the [vector store](/docs/api-reference/vector-stores/object) that the [File](/docs/api-
    /// reference/files) is attached to.
    pub vector_store_id: String,
    /// The status of the vector store files batch, which can be either `in_progress`, `completed`,
    /// `cancelled` or `failed`.
    pub status: String,
    pub file_counts: VectorStoreFileBatchObjectFileCounts,
}
