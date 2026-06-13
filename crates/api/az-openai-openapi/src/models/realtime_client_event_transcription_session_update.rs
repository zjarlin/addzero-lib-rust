// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeClientEventTranscriptionSessionUpdate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateRequest,
};

/// Send this event to update a transcription session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeClientEventTranscriptionSessionUpdate {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `transcription_session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub session: RealtimeTranscriptionSessionCreateRequest,
}
