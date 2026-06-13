// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VadConfig` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Must be set to `server_vad` to enable manual chunking using server side VAD.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Amount of audio to include before the VAD detected speech (in milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i32>,
    /// Duration of silence to detect speech stop (in milliseconds). With shorter values the model will
    /// respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i32>,
    /// Sensitivity threshold (0.0 to 1.0) for voice activity detection. A higher threshold will require
    /// louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}
