// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebSearchToolCallAction,
};

/// The results of a web search tool call. See the [web search guide](/docs/guides/tools-web-search) for
/// more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolCall {
    /// The unique ID of the web search tool call.
    pub id: String,
    /// The type of the web search tool call. Always `web_search_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The status of the web search tool call.
    pub status: String,
    /// An object describing the specific action taken in this web search call. Includes details on how the
    /// model used the web (search, open_page, find_in_page).
    pub action: WebSearchToolCallAction,
}
