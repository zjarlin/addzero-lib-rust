// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateResponseAudioInputNoiseReduction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NoiseReductionType,
};

/// Configuration for input audio noise reduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<NoiseReductionType>,
}
