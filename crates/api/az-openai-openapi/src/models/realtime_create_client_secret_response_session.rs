// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCreateClientSecretResponseSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseGA,
    RealtimeTranscriptionSessionCreateResponseGA,
};

/// The session configuration for either a realtime or transcription session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeCreateClientSecretResponseSession {
    RealtimeSessionCreateResponseGA(RealtimeSessionCreateResponseGA),
    RealtimeTranscriptionSessionCreateResponseGA(RealtimeTranscriptionSessionCreateResponseGA),
}
