// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventTranscriptionSessionCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateResponse,
};

/// Returned when a transcription session is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventTranscriptionSessionCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `transcription_session.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub session: RealtimeTranscriptionSessionCreateResponse,
}
