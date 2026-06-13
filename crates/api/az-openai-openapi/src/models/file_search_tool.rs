// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileSearchTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Filters,
    RankingOptions,
};

/// A tool that searches for relevant content from uploaded files. Learn more about the [file search
/// tool](https://platform.openai.com/docs/guides/tools-file-search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchTool {
    /// The type of the file search tool. Always `file_search`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The IDs of the vector stores to search.
    pub vector_store_ids: Vec<String>,
    /// The maximum number of results to return. This number should be between 1 and 50 inclusive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<i32>,
    /// Ranking options for search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<RankingOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<Filters>,
}
