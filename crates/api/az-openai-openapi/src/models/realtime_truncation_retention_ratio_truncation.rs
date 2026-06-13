// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTruncationRetentionRatioTruncation` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTruncationRetentionRatioTruncationTokenLimits,
};

/// Retain a fraction of the conversation tokens when the conversation exceeds the input token limit.
/// This allows you to amortize truncations across multiple turns, which can help improve cached token
/// usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTruncationRetentionRatioTruncation {
    /// Use retention ratio truncation.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Fraction of post-instruction conversation tokens to retain (`0.0` - `1.0`) when the conversation
    /// exceeds the input token limit. Setting this to `0.8` means that messages will be dropped until 80%
    /// of the maximum allowed tokens are used. This helps reduce the frequency of truncations and improve
    /// cache rates.
    pub retention_ratio: f64,
    /// Optional custom token limits for this truncation strategy. If not provided, the model's default
    /// token limits will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_limits: Option<RealtimeTruncationRetentionRatioTruncationTokenLimits>,
}
