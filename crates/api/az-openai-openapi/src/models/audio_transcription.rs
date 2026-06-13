// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AudioTranscription` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscription {
    /// The model to use for transcription. Current options are `whisper-1`, `gpt-4o-mini-transcribe`,
    /// `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe`, `gpt-4o-transcribe-diarize`, and `gpt-
    /// realtime-whisper`. Use `gpt-4o-transcribe-diarize` when you need diarization with speaker labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The language of the input audio. Supplying the input language in
    /// [ISO-639-1](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) (e.g. `en`) format will improve
    /// accuracy and latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// An optional text to guide the model's style or continue a previous audio segment. For `whisper-1`,
    /// the [prompt is a list of keywords](/docs/guides/speech-to-text#prompting). For `gpt-4o-transcribe`
    /// models (excluding `gpt-4o-transcribe-diarize`), the prompt is a free text string, for example
    /// "expect words related to technology". Prompt is not supported with `gpt-realtime-whisper` in GA
    /// Realtime sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Controls how long the model waits before emitting transcription text. Higher values can improve
    /// transcription accuracy at the cost of latency. Only supported with `gpt-realtime-whisper` in GA
    /// Realtime sessions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<String>,
}
