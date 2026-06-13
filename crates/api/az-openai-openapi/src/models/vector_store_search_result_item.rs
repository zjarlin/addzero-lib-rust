// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreSearchResultItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileAttributes,
    VectorStoreSearchResultContentObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreSearchResultItem {
    /// The ID of the vector store file.
    pub file_id: String,
    /// The name of the vector store file.
    pub filename: String,
    /// The similarity score for the result.
    pub score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
    /// Content chunks from the file.
    pub content: Vec<VectorStoreSearchResultContentObject>,
}
