// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TextContent` DTO.

use serde::{Deserialize, Serialize};

/// A text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub type_value: String,
    pub text: String,
}
