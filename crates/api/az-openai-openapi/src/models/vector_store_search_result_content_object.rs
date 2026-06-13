// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreSearchResultContentObject` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreSearchResultContentObject {
    /// The type of content.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text content returned from search.
    pub text: String,
}
