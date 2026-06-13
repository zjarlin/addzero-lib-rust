// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UrlCitationBody` DTO.

use serde::{Deserialize, Serialize};

/// A citation for a web resource used to generate a model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitationBody {
    /// The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the web resource.
    pub url: String,
    /// The index of the first character of the URL citation in the message.
    pub start_index: i32,
    /// The index of the last character of the URL citation in the message.
    pub end_index: i32,
    /// The title of the web resource.
    pub title: String,
}
