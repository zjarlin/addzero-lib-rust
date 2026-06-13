// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFormatJsonSchema` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseFormatJsonSchemaJsonSchema,
};

/// JSON Schema response format. Used to generate structured JSON responses. Learn more about
/// [Structured Outputs](/docs/guides/structured-outputs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatJsonSchema {
    /// The type of response format being defined. Always `json_schema`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Structured Outputs configuration options, including a JSON Schema.
    pub json_schema: ResponseFormatJsonSchemaJsonSchema,
}
