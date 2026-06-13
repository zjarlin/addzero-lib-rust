// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionMessageListDataItemAnnotation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionMessageListDataItemAnnotationUrlCitation,
};

/// A URL citation when using web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageListDataItemAnnotation {
    /// The type of the URL citation. Always `url_citation`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A URL citation when using web search.
    pub url_citation: ChatCompletionMessageListDataItemAnnotationUrlCitation,
}
