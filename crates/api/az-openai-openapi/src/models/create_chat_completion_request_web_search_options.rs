// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionRequestWebSearchOptions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateChatCompletionRequestWebSearchOptionsUserLocation,
    WebSearchContextSize,
};

/// This tool searches the web for relevant results to use in a response. Learn more about the [web
/// search tool](/docs/guides/tools-web-search?api-mode=chat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionRequestWebSearchOptions {
    /// Approximate location parameters for the search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_location: Option<CreateChatCompletionRequestWebSearchOptionsUserLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<WebSearchContextSize>,
}
