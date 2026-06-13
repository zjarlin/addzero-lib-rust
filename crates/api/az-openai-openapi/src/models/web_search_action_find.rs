// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchActionFind` DTO.

use serde::{Deserialize, Serialize};

/// Action type "find_in_page": Searches for a pattern within a loaded page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchActionFind {
    /// The action type.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the page searched for the pattern.
    pub url: String,
    /// The pattern or text to search for within the page.
    pub pattern: String,
}
