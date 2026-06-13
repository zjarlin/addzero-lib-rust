// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebSearchApproximateLocation,
    WebSearchToolFilters,
};

/// Search the Internet for sources related to the prompt. Learn more about the [web search
/// tool](/docs/guides/tools-web-search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    /// The type of the web search tool. One of `web_search` or `web_search_2025_08_26`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<WebSearchToolFilters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchApproximateLocation>,
    /// High level guidance for the amount of context window space to use for the search. One of `low`,
    /// `medium`, or `high`. `medium` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
}
