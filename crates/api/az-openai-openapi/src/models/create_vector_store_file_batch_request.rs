// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVectorStoreFileBatchRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChunkingStrategyRequestParam,
    CreateVectorStoreFileRequest,
    VectorStoreFileAttributes,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreFileBatchRequest {
    /// A list of [File](/docs/api-reference/files) IDs that the vector store should use. Useful for tools
    /// like `file_search` that can access files. If `attributes` or `chunking_strategy` are provided, they
    /// will be applied to all files in the batch. The maximum batch size is 2000 files. This endpoint is
    /// recommended for multi-file ingestion and helps reduce per-vector-store write request pressure.
    /// Mutually exclusive with `files`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// A list of objects that each include a `file_id` plus optional `attributes` or `chunking_strategy`.
    /// Use this when you need to override metadata for specific files. The global `attributes` or
    /// `chunking_strategy` will be ignored and must be specified for each file. The maximum batch size is
    /// 2000 files. This endpoint is recommended for multi-file ingestion and helps reduce per-vector-store
    /// write request pressure. Mutually exclusive with `file_ids`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<CreateVectorStoreFileRequest>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<ChunkingStrategyRequestParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
}
