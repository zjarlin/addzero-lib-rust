// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionUpdateRequestAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionUpdateRequestAudioInputNoiseReduction,
    RealtimeTranslationSessionUpdateRequestAudioInputTranscription,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<RealtimeTranslationSessionUpdateRequestAudioInputTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeTranslationSessionUpdateRequestAudioInputNoiseReduction>,
}
