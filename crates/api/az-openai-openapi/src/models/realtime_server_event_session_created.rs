// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventSessionCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventSessionCreatedSession,
};

/// Returned when a Session is created. Emitted automatically when a new connection is established as
/// the first server event. This event will contain the default Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventSessionCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The session configuration.
    pub session: RealtimeServerEventSessionCreatedSession,
}
