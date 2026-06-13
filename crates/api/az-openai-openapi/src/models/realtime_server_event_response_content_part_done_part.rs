// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventResponseContentPartDonePart` DTO.

use serde::{Deserialize, Serialize};

/// The content part that is done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventResponseContentPartDonePart {
    /// The content type ("text", "audio").
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content (if type is "text").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded audio data (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    /// The transcript of the audio (if type is "audio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}
