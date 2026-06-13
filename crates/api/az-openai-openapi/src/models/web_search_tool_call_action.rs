// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchToolCallAction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebSearchActionFind,
    WebSearchActionOpenPage,
    WebSearchActionSearch,
};

/// An object describing the specific action taken in this web search call. Includes details on how the
/// model used the web (search, open_page, find_in_page).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebSearchToolCallAction {
    WebSearchActionSearch(WebSearchActionSearch),
    WebSearchActionOpenPage(WebSearchActionOpenPage),
    WebSearchActionFind(WebSearchActionFind),
}
