// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeClientEventSessionUpdateSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateRequestGA,
    RealtimeTranscriptionSessionCreateRequestGA,
};

/// Update the Realtime session. Choose either a realtime session or a transcription session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeClientEventSessionUpdateSession {
    RealtimeSessionCreateRequestGA(RealtimeSessionCreateRequestGA),
    RealtimeTranscriptionSessionCreateRequestGA(RealtimeTranscriptionSessionCreateRequestGA),
}
