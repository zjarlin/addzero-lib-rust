// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFormatText` DTO.

use serde::{Deserialize, Serialize};

/// Default response format. Used to generate text responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatText {
    /// The type of response format being defined. Always `text`.
    #[serde(rename = "type")]
    pub type_value: String,
}
