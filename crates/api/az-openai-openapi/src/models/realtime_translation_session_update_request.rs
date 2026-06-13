// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionUpdateRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionUpdateRequestAudio,
};

/// Realtime translation session fields that can be updated with `session.update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequest {
    /// Configuration for translation input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeTranslationSessionUpdateRequestAudio>,
}
