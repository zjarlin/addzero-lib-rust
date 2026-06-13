// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionAudioInputNoiseReduction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NoiseReductionType,
};

/// Optional input noise reduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionAudioInputNoiseReduction {
    #[serde(rename = "type")]
    pub type_value: NoiseReductionType,
}
