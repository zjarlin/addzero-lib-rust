// Generated from OpenAPI spec. Do not edit by hand.
//! `SpeechAudioDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    SpeechAudioDoneEventUsage,
};

/// Emitted when the speech synthesis is complete and all audio has been streamed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAudioDoneEvent {
    /// The type of the event. Always `speech.audio.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Token usage statistics for the request.
    pub usage: SpeechAudioDoneEventUsage,
}
