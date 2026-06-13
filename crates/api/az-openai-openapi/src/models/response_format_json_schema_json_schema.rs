// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFormatJsonSchemaJsonSchema` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseFormatJsonSchemaSchema,
};

/// Structured Outputs configuration options, including a JSON Schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatJsonSchemaJsonSchema {
    /// A description of what the response format is for, used by the model to determine how to respond in
    /// the format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a
    /// maximum length of 64.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<ResponseFormatJsonSchemaSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
