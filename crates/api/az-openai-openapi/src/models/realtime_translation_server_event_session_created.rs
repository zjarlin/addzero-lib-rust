// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSession,
};

/// Returned when a translation session is created. Emitted automatically when a new connection is
/// established as the first server event. This event contains the default translation session
/// configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The translation session configuration.
    pub session: RealtimeTranslationSession,
}
