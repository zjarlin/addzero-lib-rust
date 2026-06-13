// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateResponseGAAudioInputNoiseReduction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NoiseReductionType,
};

/// Configuration for input audio noise reduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGAAudioInputNoiseReduction {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<NoiseReductionType>,
}
