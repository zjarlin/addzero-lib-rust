// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeAudioFormatsPCMAudioFormat` DTO.

use serde::{Deserialize, Serialize};

/// The PCM audio format. Only a 24kHz sample rate is supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAudioFormatsPCMAudioFormat {
    /// The audio format. Always `audio/pcm`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The sample rate of the audio. Always `24000`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<i32>,
}
