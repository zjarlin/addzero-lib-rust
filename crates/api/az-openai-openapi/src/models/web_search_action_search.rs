// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchActionSearch` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebSearchActionSearchSource,
};

/// Action type "search" - Performs a web search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchActionSearch {
    /// The action type.
    #[serde(rename = "type")]
    pub type_value: String,
    /// [DEPRECATED] The search query.
    pub query: String,
    /// The search queries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<String>>,
    /// The sources used in the search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<WebSearchActionSearchSource>>,
}
