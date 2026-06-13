// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponseVerboseJson` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextUsageDuration,
    TranscriptionSegment,
    TranscriptionWord,
};

/// Represents a verbose json transcription response returned by model, based on the provided input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranscriptionResponseVerboseJson {
    /// The language of the input audio.
    pub language: String,
    /// The duration of the input audio.
    pub duration: f64,
    /// The transcribed text.
    pub text: String,
    /// Extracted words and their corresponding timestamps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<TranscriptionWord>>,
    /// Segments of the transcribed text and their corresponding details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<TranscriptionSegment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptTextUsageDuration>,
}
