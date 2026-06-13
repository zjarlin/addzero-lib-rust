// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreSearchRequestRankingOptions` DTO.

use serde::{Deserialize, Serialize};

/// Ranking options for search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreSearchRequestRankingOptions {
    /// Enable re-ranking; set to `none` to disable, which can help reduce latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
}
