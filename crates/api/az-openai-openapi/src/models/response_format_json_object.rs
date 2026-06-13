// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseFormatJsonObject` DTO.

use serde::{Deserialize, Serialize};

/// JSON object response format. An older method of generating JSON responses. Using `json_schema` is
/// recommended for models that support it. Note that the model will not generate JSON without a system
/// or user message instructing it to do so.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatJsonObject {
    /// The type of response format being defined. Always `json_object`.
    #[serde(rename = "type")]
    pub type_value: String,
}
