// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchActionOpenPage` DTO.

use serde::{Deserialize, Serialize};

/// Action type "open_page" - Opens a specific URL from search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchActionOpenPage {
    /// The action type.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL opened by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
