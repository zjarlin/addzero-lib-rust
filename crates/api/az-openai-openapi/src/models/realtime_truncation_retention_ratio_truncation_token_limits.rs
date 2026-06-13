// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTruncationRetentionRatioTruncationTokenLimits` DTO.

use serde::{Deserialize, Serialize};

/// Optional custom token limits for this truncation strategy. If not provided, the model's default
/// token limits will be used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTruncationRetentionRatioTruncationTokenLimits {
    /// Maximum tokens allowed in the conversation after instructions (which including tool definitions).
    /// For example, setting this to 5,000 would mean that truncation would occur when the conversation
    /// exceeds 5,000 tokens after instructions. This cannot be higher than the model's context window size
    /// minus the maximum output tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_instructions: Option<i32>,
}
