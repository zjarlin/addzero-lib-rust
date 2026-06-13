// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionUpdateRequestAudioOutput` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioOutput {
    /// Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
