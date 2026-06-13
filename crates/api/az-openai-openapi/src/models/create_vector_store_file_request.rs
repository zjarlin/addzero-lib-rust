// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVectorStoreFileRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChunkingStrategyRequestParam,
    VectorStoreFileAttributes,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreFileRequest {
    /// A [File](/docs/api-reference/files) ID that the vector store should use. Useful for tools like
    /// `file_search` that can access files. For multi-file ingestion, we recommend
    /// [`file_batches`](/docs/api-reference/vector-stores-file-batches/createBatch) to minimize per-vector-
    /// store write requests.
    pub file_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<ChunkingStrategyRequestParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
}
