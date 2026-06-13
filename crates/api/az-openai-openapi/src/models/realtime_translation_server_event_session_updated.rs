// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationServerEventSessionUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSession,
};

/// Returned when a translation session is updated with a `session.update` event, unless there is an
/// error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationServerEventSessionUpdated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The translation session configuration.
    pub session: RealtimeTranslationSession,
}
