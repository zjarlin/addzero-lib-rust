// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreFileObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileAttributes,
    VectorStoreFileObjectChunkingStrategy,
    VectorStoreFileObjectLastError,
};

/// A list of files attached to a vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileObject {
    /// The identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `vector_store.file`.
    pub object: String,
    /// The total vector store usage in bytes. Note that this may be different from the original file size.
    pub usage_bytes: i32,
    /// The Unix timestamp (in seconds) for when the vector store file was created.
    pub created_at: i64,
    /// The ID of the [vector store](/docs/api-reference/vector-stores/object) that the [File](/docs/api-
    /// reference/files) is attached to.
    pub vector_store_id: String,
    /// The status of the vector store file, which can be either `in_progress`, `completed`, `cancelled`, or
    /// `failed`. The status `completed` indicates that the vector store file is ready for use.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<VectorStoreFileObjectLastError>,
    /// The strategy used to chunk the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<VectorStoreFileObjectChunkingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
}
