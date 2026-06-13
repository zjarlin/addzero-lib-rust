// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseUsageInputTokenDetailsCachedTokensDetails` DTO.

use serde::{Deserialize, Serialize};

/// Details about the cached tokens used as input for the Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseUsageInputTokenDetailsCachedTokensDetails {
    /// The number of cached text tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i32>,
    /// The number of cached image tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<i32>,
    /// The number of cached audio tokens used as input for the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,
}
