// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreFileContentResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileContentResponseDataItem,
};

/// Represents the parsed content of a vector store file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileContentResponse {
    /// The object type, which is always `vector_store.file_content.page`
    pub object: String,
    /// Parsed content of the file.
    pub data: Vec<VectorStoreFileContentResponseDataItem>,
    /// Indicates if there are more content pages to fetch.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}
