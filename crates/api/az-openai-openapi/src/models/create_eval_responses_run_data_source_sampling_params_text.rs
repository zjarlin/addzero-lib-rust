// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceSamplingParamsText` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TextResponseFormatConfiguration,
};

/// Configuration options for a text response from the model. Can be plain text or structured JSON data.
/// Learn more: - [Text inputs and outputs](/docs/guides/text) - [Structured
/// Outputs](/docs/guides/structured-outputs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalResponsesRunDataSourceSamplingParamsText {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<TextResponseFormatConfiguration>,
}
