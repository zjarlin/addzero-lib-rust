// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventSessionCreatedSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseGA,
    RealtimeTranscriptionSessionCreateResponseGA,
};

/// The session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeServerEventSessionCreatedSession {
    RealtimeSessionCreateResponseGA(RealtimeSessionCreateResponseGA),
    RealtimeTranscriptionSessionCreateResponseGA(RealtimeTranscriptionSessionCreateResponseGA),
}
