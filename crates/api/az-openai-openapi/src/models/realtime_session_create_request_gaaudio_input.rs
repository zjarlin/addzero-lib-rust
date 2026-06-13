// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeSessionCreateRequestGAAudioInput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AudioTranscription,
    RealtimeAudioFormats,
    RealtimeSessionCreateRequestGAAudioInputNoiseReduction,
    RealtimeTurnDetection,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateRequestGAAudioInput {
    /// The format of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// Configuration for input audio transcription, defaults to off and can be set to `null` to turn off
    /// once on. Input audio transcription is not native to the model, since the model consumes audio
    /// directly. Transcription runs asynchronously through [the /audio/transcriptions endpoint](/docs/api-
    /// reference/audio/createTranscription) and should be treated as guidance of input audio content rather
    /// than precisely what the model heard. The client can optionally set the language and prompt for
    /// transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcription: Option<AudioTranscription>,
    /// Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise
    /// reduction filters audio added to the input audio buffer before it is sent to VAD and the model.
    /// Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model
    /// performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<RealtimeSessionCreateRequestGAAudioInputNoiseReduction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeTurnDetection>,
}
