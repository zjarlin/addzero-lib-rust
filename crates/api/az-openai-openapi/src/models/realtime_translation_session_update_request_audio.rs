// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionUpdateRequestAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionUpdateRequestAudioInput,
    RealtimeTranslationSessionUpdateRequestAudioOutput,
};

/// Configuration for translation input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeTranslationSessionUpdateRequestAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeTranslationSessionUpdateRequestAudioOutput>,
}
