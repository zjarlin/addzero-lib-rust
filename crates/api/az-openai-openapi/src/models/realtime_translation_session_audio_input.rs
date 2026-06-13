// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionAudioInputNoiseReduction,
    RealtimeTranslationSessionAudioInputTranscription,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<RealtimeTranslationSessionAudioInputTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeTranslationSessionAudioInputNoiseReduction>,
}
