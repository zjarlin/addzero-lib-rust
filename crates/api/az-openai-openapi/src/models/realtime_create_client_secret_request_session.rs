// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCreateClientSecretRequestSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateRequestGA,
    RealtimeTranscriptionSessionCreateRequestGA,
};

/// Session configuration to use for the client secret. Choose either a realtime session or a
/// transcription session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeCreateClientSecretRequestSession {
    RealtimeSessionCreateRequestGA(RealtimeSessionCreateRequestGA),
    RealtimeTranscriptionSessionCreateRequestGA(RealtimeTranscriptionSessionCreateRequestGA),
}
