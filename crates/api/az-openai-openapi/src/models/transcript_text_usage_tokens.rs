// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TranscriptTextUsageTokens` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageTokensInputTokenDetails,
};

/// Usage statistics for models billed by token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextUsageTokens {
    /// The type of the usage object. Always `tokens` for this variant.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Number of input tokens billed for this request.
    pub input_tokens: i32,
    /// Details about the input tokens billed for this request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<TranscriptTextUsageTokensInputTokenDetails>,
    /// Number of output tokens generated.
    pub output_tokens: i32,
    /// Total number of tokens used (input + output).
    pub total_tokens: i32,
}
