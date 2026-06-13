// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventSessionUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeServerEventSessionUpdatedSession,
};

/// Returned when a session is updated with a `session.update` event, unless there is an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventSessionUpdated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The session configuration.
    pub session: RealtimeServerEventSessionUpdatedSession,
}
