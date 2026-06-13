// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeServerEventSessionUpdatedSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseGA,
    RealtimeTranscriptionSessionCreateResponseGA,
};

/// The session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventSessionUpdatedSession {
    RealtimeSessionCreateResponseGA(RealtimeSessionCreateResponseGA),
    RealtimeTranscriptionSessionCreateResponseGA(RealtimeTranscriptionSessionCreateResponseGA),
}
