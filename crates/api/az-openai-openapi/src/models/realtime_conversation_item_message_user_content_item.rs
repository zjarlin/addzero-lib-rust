// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeConversationItemMessageUserContentItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConversationItemMessageUserContentItem {
    /// The content type (`input_text`, `input_audio`, or `input_image`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content (for `input_text`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded audio bytes (for `input_audio`), these will be parsed as the format specified in the
    /// session input audio type configuration. This defaults to PCM 16-bit 24kHz mono if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    /// Base64-encoded image bytes (for `input_image`) as a data URI. For example
    /// `data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...`. Supported formats are PNG and JPEG.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// The detail level of the image (for `input_image`). `auto` will default to `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Transcript of the audio (for `input_audio`). This is not sent to the model, but will be attached to
    /// the message item for reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}
