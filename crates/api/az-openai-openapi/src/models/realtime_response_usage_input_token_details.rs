// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseUsageInputTokenDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeResponseUsageInputTokenDetailsCachedTokensDetails,
};

/// Details about the input tokens used in the Response. Cached tokens are tokens from previous turns in
/// the conversation that are included as context for the current response. Cached tokens here are
/// counted as a subset of input tokens, meaning input tokens will include cached and uncached tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseUsageInputTokenDetails {
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
    pub cached_tokens_details: Option<RealtimeResponseUsageInputTokenDetailsCachedTokensDetails>,
}
