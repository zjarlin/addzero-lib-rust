// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    NoiseReductionType,
};

/// Optional input noise reduction. Set to `null` to disable it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction {
    #[serde(rename = "type")]
    pub type_value: NoiseReductionType,
}
