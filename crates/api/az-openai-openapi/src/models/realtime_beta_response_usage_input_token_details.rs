// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaResponseUsageInputTokenDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaResponseUsageInputTokenDetailsCachedTokensDetails,
};

/// Details about the input tokens used in the Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaResponseUsageInputTokenDetails {
    /// The number of cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<i32>,
    /// The number of text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i32>,
    /// The number of image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<i32>,
    /// The number of audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,
    /// Details about the cached tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens_details: Option<RealtimeBetaResponseUsageInputTokenDetailsCachedTokensDetails>,
}
