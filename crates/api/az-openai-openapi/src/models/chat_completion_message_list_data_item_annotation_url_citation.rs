// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionMessageListDataItemAnnotationUrlCitation` DTO.

use serde::{Deserialize, Serialize};

/// A URL citation when using web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionMessageListDataItemAnnotationUrlCitation {
    /// The index of the last character of the URL citation in the message.
    pub end_index: i32,
    /// The index of the first character of the URL citation in the message.
    pub start_index: i32,
    /// The URL of the web resource.
    pub url: String,
    /// The title of the web resource.
    pub title: String,
}
