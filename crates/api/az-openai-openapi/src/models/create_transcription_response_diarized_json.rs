// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponseDiarizedJson` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateTranscriptionResponseDiarizedJsonUsage,
    TranscriptionDiarizedSegment,
};

/// Represents a diarized transcription response returned by the model, including the combined
/// transcript and speaker-segment annotations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranscriptionResponseDiarizedJson {
    /// The type of task that was run. Always `transcribe`.
    pub task: String,
    /// Duration of the input audio in seconds.
    pub duration: f64,
    /// The concatenated transcript text for the entire audio input.
    pub text: String,
    /// Segments of the transcript annotated with timestamps and speaker labels.
    pub segments: Vec<TranscriptionDiarizedSegment>,
    /// Token or duration usage statistics for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CreateTranscriptionResponseDiarizedJsonUsage>,
}
