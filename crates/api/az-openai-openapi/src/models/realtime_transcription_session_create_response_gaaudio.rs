// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateResponseGAAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateResponseGAAudioInput,
};

/// Configuration for input audio for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGAAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeTranscriptionSessionCreateResponseGAAudioInput>,
}
