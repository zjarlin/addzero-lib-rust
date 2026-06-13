// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionClosed` DTO.

use serde::{Deserialize, Serialize};

/// Returned when a realtime translation session is closed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionClosed {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.closed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
