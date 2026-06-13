// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreSearchResultsPage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreSearchResultItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreSearchResultsPage {
    /// The object type, which is always `vector_store.search_results.page`
    pub object: String,
    pub search_query: Vec<String>,
    /// The list of search result items.
    pub data: Vec<VectorStoreSearchResultItem>,
    /// Indicates if there are more results to fetch.
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}
