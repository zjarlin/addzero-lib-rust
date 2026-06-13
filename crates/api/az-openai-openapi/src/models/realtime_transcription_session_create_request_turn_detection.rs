// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateRequestTurnDetection` DTO.

use serde::{Deserialize, Serialize};

/// Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model
/// will detect the start and end of speech based on audio volume and respond at the end of user speech.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequestTurnDetection {
    /// Type of turn detection. Only `server_vad` is currently supported for transcription sessions.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A higher threshold will require
    /// louder audio to activate the model, and thus might perform better in noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Amount of audio to include before the VAD detected speech (in milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i32>,
    /// Duration of silence to detect speech stop (in milliseconds). Defaults to 500ms. With shorter values
    /// the model will respond more quickly, but may jump in on short pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i32>,
}
