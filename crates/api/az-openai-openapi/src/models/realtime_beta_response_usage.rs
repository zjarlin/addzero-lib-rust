// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaResponseUsage` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaResponseUsageInputTokenDetails,
    RealtimeBetaResponseUsageOutputTokenDetails,
};

/// Usage statistics for the Response, this will correspond to billing. A Realtime API session will
/// maintain a conversation context and append new Items to the Conversation, thus output from previous
/// turns (text and audio tokens) will become the input for later turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaResponseUsage {
    /// The total number of tokens in the Response including input and output text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i32>,
    /// The number of input tokens used in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<i32>,
    /// The number of output tokens sent in the Response, including text and audio tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<i32>,
    /// Details about the input tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<RealtimeBetaResponseUsageInputTokenDetails>,
    /// Details about the output tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_token_details: Option<RealtimeBetaResponseUsageOutputTokenDetails>,
}
