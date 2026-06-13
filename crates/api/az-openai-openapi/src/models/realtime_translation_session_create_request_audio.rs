// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionCreateRequestAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionCreateRequestAudioInput,
    RealtimeTranslationSessionCreateRequestAudioOutput,
};

/// Configuration for translation input and output audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudio {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RealtimeTranslationSessionCreateRequestAudioInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<RealtimeTranslationSessionCreateRequestAudioOutput>,
}
