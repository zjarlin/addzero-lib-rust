// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeConversationItemMessageSystemContentItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConversationItemMessageSystemContentItem {
    /// The content type. Always `input_text` for system messages.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The text content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
