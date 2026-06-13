// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TextResponseFormatConfiguration` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseFormatJsonObject,
    ResponseFormatText,
    TextResponseFormatJsonSchema,
};

/// An object specifying the format that the model must output. Configuring `{ "type": "json_schema" }`
/// enables Structured Outputs, which ensures the model will match your supplied JSON schema. Learn more
/// in the [Structured Outputs guide](/docs/guides/structured-outputs). The default format is `{ "type":
/// "text" }` with no additional options. **Not recommended for gpt-4o and newer models:** Setting to `{
/// "type": "json_object" }` enables the older JSON mode, which ensures the message the model generates
/// is valid JSON. Using `json_schema` is preferred for models that support it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextResponseFormatConfiguration {
    ResponseFormatText(ResponseFormatText),
    TextResponseFormatJsonSchema(TextResponseFormatJsonSchema),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
}
