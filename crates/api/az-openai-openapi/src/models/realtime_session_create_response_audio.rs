// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseAudioInput,
    RealtimeSessionCreateResponseAudioOutput,
};

/// Configuration for input and output audio for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeSessionCreateResponseAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeSessionCreateResponseAudioOutput>,
}
