// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCallCreateRequestSessionAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeCallCreateRequestSessionAudioInput,
    RealtimeCallCreateRequestSessionAudioOutput,
};

/// Configuration for input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCallCreateRequestSessionAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeCallCreateRequestSessionAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeCallCreateRequestSessionAudioOutput>,
}
