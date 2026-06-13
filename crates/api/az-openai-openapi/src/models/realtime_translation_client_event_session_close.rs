// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationClientEventSessionClose` DTO.

use serde::{Deserialize, Serialize};

/// Gracefully close the realtime translation session. The server flushes pending input audio and emits
/// any remaining translated output before closing the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationClientEventSessionClose {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `session.close`.
    #[serde(rename = "type")]
    pub type_value: String,
}
