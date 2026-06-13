// Generated from OpenAPI spec. Do not edit by hand.
//! `WebSearchPreviewTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ApproximateLocation,
    SearchContentType,
    SearchContextSize,
};

/// This tool searches the web for relevant results to use in a response. Learn more about the [web
/// search tool](https://platform.openai.com/docs/guides/tools-web-search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchPreviewTool {
    /// The type of the web search tool. One of `web_search_preview` or `web_search_preview_2025_03_11`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: Option<ApproximateLocation>,
    /// High level guidance for the amount of context window space to use for the search. One of `low`,
    /// `medium`, or `high`. `medium` is the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<SearchContextSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_content_types: Option<Vec<SearchContentType>>,
}
