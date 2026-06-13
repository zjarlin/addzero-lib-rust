// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionCreateRequestAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction,
    RealtimeTranslationSessionCreateRequestAudioInputTranscription,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<RealtimeTranslationSessionCreateRequestAudioInputTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeTranslationSessionCreateRequestAudioInputNoiseReduction>,
}
