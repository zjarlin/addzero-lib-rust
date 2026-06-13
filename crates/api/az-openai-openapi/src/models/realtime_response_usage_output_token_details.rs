// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseUsageOutputTokenDetails` DTO.

use serde::{Deserialize, Serialize};

/// Details about the output tokens used in the Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseUsageOutputTokenDetails {
    /// The number of text tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<i32>,
    /// The number of audio tokens used in the Response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,
}
