// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationClientEventSessionUpdate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionUpdateRequest,
};

/// Send this event to update the translation session configuration. Translation sessions support
/// updates to `audio.output.language`, `audio.input.transcription`, and `audio.input.noise_reduction`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationClientEventSessionUpdate {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Translation session fields to update. The session `type` and `model` are set at creation and cannot
    /// be changed with `session.update`.
    pub session: RealtimeTranslationSessionUpdateRequest,
}
