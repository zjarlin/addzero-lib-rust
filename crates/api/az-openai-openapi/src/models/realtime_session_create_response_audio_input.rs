// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AudioTranscriptionResponse,
    RealtimeAudioFormats,
    RealtimeSessionCreateResponseAudioInputNoiseReduction,
    RealtimeSessionCreateResponseAudioInputTurnDetection,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseAudioInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// Configuration for input audio transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<AudioTranscriptionResponse>,
    /// Configuration for input audio noise reduction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeSessionCreateResponseAudioInputNoiseReduction>,
    /// Configuration for turn detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeSessionCreateResponseAudioInputTurnDetection>,
}
