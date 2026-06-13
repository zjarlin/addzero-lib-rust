// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventTranscriptionSessionUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateResponse,
};

/// Returned when a transcription session is updated with a `transcription_session.update` event, unless
/// there is an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventTranscriptionSessionUpdated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `transcription_session.updated`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub session: RealtimeTranscriptionSessionCreateResponse,
}
