// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionAudioOutput` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionAudioOutput {
    /// Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
