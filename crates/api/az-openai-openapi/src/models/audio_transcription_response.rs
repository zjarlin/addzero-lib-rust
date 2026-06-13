// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AudioTranscriptionResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    /// The model used for transcription. Current options are `whisper-1`, `gpt-4o-mini-transcribe`,
    /// `gpt-4o-mini-transcribe-2025-12-15`, `gpt-4o-transcribe`, `gpt-4o-transcribe-diarize`, and `gpt-
    /// realtime-whisper`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The language of the input audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The prompt configured for input audio transcription, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}
