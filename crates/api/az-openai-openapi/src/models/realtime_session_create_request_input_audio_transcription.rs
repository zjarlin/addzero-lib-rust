// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateRequestInputAudioTranscription` DTO.

use serde::{Deserialize, Serialize};

/// Configuration for input audio transcription, defaults to off and can be set to `null` to turn off
/// once on. Input audio transcription is not native to the model, since the model consumes audio
/// directly. Transcription runs asynchronously and should be treated as rough guidance rather than the
/// representation understood by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateRequestInputAudioTranscription {
    /// The model to use for transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
