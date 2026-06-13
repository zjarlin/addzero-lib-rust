// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionResponseMessageAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionResponseMessageAnnotationUrlCitation,
};

/// A URL citation when using web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponseMessageAnnotation {
    /// The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A URL citation when using web search.
    pub url_citation: ChatCompletionResponseMessageAnnotationUrlCitation,
}
