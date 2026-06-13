// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionAudioInput,
    RealtimeTranslationSessionAudioOutput,
};

/// Configuration for translation input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeTranslationSessionAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeTranslationSessionAudioOutput>,
}
