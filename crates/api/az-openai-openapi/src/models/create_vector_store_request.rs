// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateVectorStoreRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateVectorStoreRequestChunkingStrategy,
    Metadata,
    VectorStoreExpirationAfter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreRequest {
    /// A list of [File](/docs/api-reference/files) IDs that the vector store should use. Useful for tools
    /// like `file_search` that can access files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    /// The name of the vector store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A description for the vector store. Can be used to describe the vector store's purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<VectorStoreExpirationAfter>,
    /// The chunking strategy used to chunk the file(s). If not set, will use the `auto` strategy. Only
    /// applicable if `file_ids` is non-empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<CreateVectorStoreRequestChunkingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
