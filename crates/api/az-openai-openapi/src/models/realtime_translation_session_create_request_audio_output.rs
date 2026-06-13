// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionCreateRequestAudioOutput` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionCreateRequestAudioOutput {
    /// Target language for translated output audio and transcript deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
