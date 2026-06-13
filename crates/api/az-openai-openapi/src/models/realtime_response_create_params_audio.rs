// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseCreateParamsAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeResponseCreateParamsAudioOutput,
};

/// Configuration for audio input and output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseCreateParamsAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeResponseCreateParamsAudioOutput>,
}
