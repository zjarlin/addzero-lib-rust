// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranscriptionSessionCreateResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonValue,
};

use crate::models::{
    AudioTranscriptionResponse,
    RealtimeTranscriptionSessionCreateResponseClientSecret,
    RealtimeTranscriptionSessionCreateResponseTurnDetection,
};

/// A new Realtime transcription session configuration. When a session is created on the server via REST
/// API, the session object also contains an ephemeral key. Default TTL for keys is 10 minutes. This
/// property is not present when a session is updated via the WebSocket API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranscriptionSessionCreateResponse {
    /// Ephemeral key returned by the API. Only present when the session is created on the server via REST
    /// API.
    pub client_secret: RealtimeTranscriptionSessionCreateResponseClientSecret,
    /// The set of modalities the model can respond with. To disable audio, set this to ["text"].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<OpenAiJsonValue>,
    /// The format of input audio. Options are `pcm16`, `g711_ulaw`, or `g711_alaw`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<String>,
    /// Configuration of the transcription model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<AudioTranscriptionResponse>,
    /// Configuration for turn detection. Can be set to `null` to turn off. Server VAD means that the model
    /// will detect the start and end of speech based on audio volume and respond at the end of user speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<RealtimeTranscriptionSessionCreateResponseTurnDetection>,
}
