// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateResponseGAAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AudioTranscriptionResponse,
    RealtimeAudioFormats,
    RealtimeTranscriptionSessionCreateResponseGAAudioInputNoiseReduction,
    RealtimeTranscriptionSessionCreateResponseGAAudioInputTurnDetection,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponseGAAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// Configuration of the transcription model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<AudioTranscriptionResponse>,
    /// Configuration for input audio noise reduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeTranscriptionSessionCreateResponseGAAudioInputNoiseReduction>,
    /// Configuration for turn detection. For `gpt-realtime-whisper`, this must be `null`; VAD is not
    /// supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeTranscriptionSessionCreateResponseGAAudioInputTurnDetection>,
}
