// Generated from OpenAPI spec. Do not edit by hand.
//! `WebSearchActionSearchSource` DTO.

use serde::{Deserialize, Serialize};

/// A source used in the search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchActionSearchSource {
    /// The type of source. Always `url`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the source.
    pub url: String,
}
