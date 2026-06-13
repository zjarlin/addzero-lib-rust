// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AudioTranscription,
    RealtimeTranscriptionSessionCreateRequestInputAudioNoiseReduction,
    RealtimeTranscriptionSessionCreateRequestTurnDetection,
};

/// Realtime transcription session object configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateRequest {
    /// Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model
    /// will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeTranscriptionSessionCreateRequestTurnDetection>,
    /// Configuration for input audio noise reduction. This can be set to `null` to turn off. Noise
    /// reduction filters audio added to the input audio buffer before it is sent to VAD and the model.
    /// Filtering the audio can improve VAD and turn detection accuracy (reducing false positives) and model
    /// performance by improving perception of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: Option<RealtimeTranscriptionSessionCreateRequestInputAudioNoiseReduction>,
    /// The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`. For `pcm16`, input
    /// audio must be 16-bit PCM at a 24kHz sample rate, single channel (mono), and little-endian byte
    /// order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<String>,
    /// Configuration for input audio transcription. The client can optionally set the language and prompt
    /// for transcription, these offer additional guidance to the transcription service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<AudioTranscription>,
    /// The set of items to include in the transcription. Current available items are:
    /// `item.input_audio_transcription.logprobs`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}
