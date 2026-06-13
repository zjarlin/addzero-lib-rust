// Generated from OpenAPI spec. Do not edit by hand.
//! `BatchUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    BatchUsageInputTokensDetails,
    BatchUsageOutputTokensDetails,
};

/// Represents token usage details including input tokens, output tokens, a breakdown of output tokens,
/// and the total tokens used. Only populated on batches created after September 7, 2025.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUsage {
    /// The number of input tokens.
    pub input_tokens: i32,
    /// A detailed breakdown of the input tokens.
    pub input_tokens_details: BatchUsageInputTokensDetails,
    /// The number of output tokens.
    pub output_tokens: i32,
    /// A detailed breakdown of the output tokens.
    pub output_tokens_details: BatchUsageOutputTokensDetails,
    /// The total number of tokens used.
    pub total_tokens: i32,
}
