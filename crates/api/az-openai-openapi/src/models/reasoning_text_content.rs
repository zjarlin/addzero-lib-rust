// Generated from OpenAPI spec. Do not edit by hand.
//! `ReasoningTextContent` DTO.

use serde::{Deserialize, Serialize};

/// Reasoning text from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTextContent {
    /// The type of the reasoning text. Always `reasoning_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The reasoning text from the model.
    pub text: String,
}
