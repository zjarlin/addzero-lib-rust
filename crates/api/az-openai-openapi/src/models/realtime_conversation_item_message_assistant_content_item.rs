// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeConversationItemMessageAssistantContentItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConversationItemMessageAssistantContentItem {
    /// The content type, `output_text` or `output_audio` depending on the session `output_modalities`
    /// configuration.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded audio bytes, these will be parsed as the format specified in the session output audio
    /// type configuration. This defaults to PCM 16-bit 24kHz mono if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    /// The transcript of the audio content, this will always be present if the output type is `audio`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}
