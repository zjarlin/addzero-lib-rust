// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OutputAudio` DTO.

use serde::{Deserialize, Serialize};

/// An audio output from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAudio {
    /// The type of the output audio. Always `output_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Base64-encoded audio data from the model.
    pub data: String,
    /// The transcript of the audio data from the model.
    pub transcript: String,
}
