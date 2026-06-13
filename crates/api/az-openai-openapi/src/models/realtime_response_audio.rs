// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeResponseAudioOutput,
};

/// Configuration for audio output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeResponseAudioOutput>,
}
