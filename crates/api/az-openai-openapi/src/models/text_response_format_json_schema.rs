// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TextResponseFormatJsonSchema` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseFormatJsonSchemaSchema,
};

/// JSON Schema response format. Used to generate structured JSON responses. Learn more about
/// [Structured Outputs](/docs/guides/structured-outputs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResponseFormatJsonSchema {
    /// The type of response format being defined. Always `json_schema`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A description of what the response format is for, used by the model to determine how to respond in
    /// the format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a
    /// maximum length of 64.
    pub name: String,
    pub schema: ResponseFormatJsonSchemaSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
