// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InputTextContent` DTO.

use serde::{Deserialize, Serialize};

/// A text input to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputTextContent {
    /// The type of the input item. Always `input_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text input to the model.
    pub text: String,
}
