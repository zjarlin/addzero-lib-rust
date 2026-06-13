// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionInputAudioNoiseReduction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NoiseReductionType,
};

/// Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise
/// reduction filters audio added to the input audio buffer before it is sent to VAD and the model.
/// Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model
/// performance by improving perception of the input audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionInputAudioNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<NoiseReductionType>,
}
