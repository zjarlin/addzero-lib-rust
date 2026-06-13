// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RankingOptions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    HybridSearchOptions,
    RankerVersionType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingOptions {
    /// The ranker to use for the file search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: Option<RankerVersionType>,
    /// The score threshold for the file search, a number between 0 and 1. Numbers closer to 1 will attempt
    /// to return only the most relevant results, but may return fewer results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
    /// Weights that control how reciprocal rank fusion balances semantic embedding matches versus sparse
    /// keyword matches when hybrid search is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hybrid_search: Option<HybridSearchOptions>,
}
