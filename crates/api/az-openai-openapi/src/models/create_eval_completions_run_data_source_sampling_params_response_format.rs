// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseFormatJsonObject,
    ResponseFormatJsonSchema,
    ResponseFormatText,
};

/// An object specifying the format that the model must output. Setting to `{ "type": "json_schema",
/// "json_schema": {...} }` enables Structured Outputs which ensures the model will match your supplied
/// JSON schema. Learn more in the [Structured Outputs guide](/docs/guides/structured-outputs). Setting
/// to `{ "type": "json_object" }` enables the older JSON mode, which ensures the message the model
/// generates is valid JSON. Using `json_schema` is preferred for models that support it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceSamplingParamsResponseFormat {
    ResponseFormatText(ResponseFormatText),
    ResponseFormatJsonSchema(ResponseFormatJsonSchema),
    ResponseFormatJsonObject(ResponseFormatJsonObject),
}
