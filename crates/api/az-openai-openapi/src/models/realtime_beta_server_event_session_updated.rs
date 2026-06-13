// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventSessionUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSession,
};

/// Returned when a session is updated with a `session.update` event, unless there is an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventSessionUpdated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub session: RealtimeSession,
}
