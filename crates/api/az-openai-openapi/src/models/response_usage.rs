// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseUsageInputTokensDetails,
    ResponseUsageOutputTokensDetails,
};

/// Represents token usage details including input tokens, output tokens, a breakdown of output tokens,
/// and the total tokens used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUsage {
    /// The number of input tokens.
    pub input_tokens: i32,
    /// A detailed breakdown of the input tokens.
    pub input_tokens_details: ResponseUsageInputTokensDetails,
    /// The number of output tokens.
    pub output_tokens: i32,
    /// A detailed breakdown of the output tokens.
    pub output_tokens_details: ResponseUsageOutputTokensDetails,
    /// The total number of tokens used.
    pub total_tokens: i32,
}
