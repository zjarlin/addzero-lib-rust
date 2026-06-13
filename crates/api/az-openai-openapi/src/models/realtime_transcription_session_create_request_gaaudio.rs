// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateRequestGAAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranscriptionSessionCreateRequestGAAudioInput,
};

/// Configuration for input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestGAAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeTranscriptionSessionCreateRequestGAAudioInput>,
}
