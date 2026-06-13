// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeConversationItemWithReferenceContentItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConversationItemWithReferenceContentItem {
    /// The content type (`input_text`, `input_audio`, `item_reference`, `text`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content, used for `input_text` and `text` content types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// ID of a previous conversation item to reference (for `item_reference` content types in
    /// `response.create` events). These can reference both client and server created items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Base64-encoded audio bytes, used for `input_audio` content type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    /// The transcript of the audio, used for `input_audio` content type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}
