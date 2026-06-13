// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebSearchToolFilters` DTO.

use serde::{Deserialize, Serialize};

/// Filters for the search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchToolFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}
