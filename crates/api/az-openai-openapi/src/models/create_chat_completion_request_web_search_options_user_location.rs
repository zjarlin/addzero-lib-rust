// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionRequestWebSearchOptionsUserLocation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebSearchLocation,
};

/// Approximate location parameters for the search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionRequestWebSearchOptionsUserLocation {
    /// The type of location approximation. Always `approximate`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub approximate: WebSearchLocation,
}
