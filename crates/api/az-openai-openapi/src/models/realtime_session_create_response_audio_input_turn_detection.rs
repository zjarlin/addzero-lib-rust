// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateResponseAudioInputTurnDetection` DTO.

use serde::{Deserialize, Serialize};

/// Configuration for turn detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInputTurnDetection {
    /// Type of turn detection, only `server_vad` is currently supported.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i32>,
}
