// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreSearchRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreSearchRequestFilters,
    VectorStoreSearchRequestQuery,
    VectorStoreSearchRequestRankingOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreSearchRequest {
    /// A query string for a search
    pub query: VectorStoreSearchRequestQuery,
    /// Whether to rewrite the natural language query for vector search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_query: Option<bool>,
    /// The maximum number of results to return. This number should be between 1 and 50 inclusive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<i32>,
    /// A filter to apply based on file attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<VectorStoreSearchRequestFilters>,
    /// Ranking options for search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<VectorStoreSearchRequestRankingOptions>,
}
