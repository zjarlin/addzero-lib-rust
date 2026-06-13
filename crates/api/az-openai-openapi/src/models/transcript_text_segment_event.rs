// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptTextSegmentEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a diarized transcription returns a completed segment with speaker information. Only
/// emitted when you [create a transcription](/docs/api-reference/audio/create-transcription) with
/// `stream` set to `true` and `response_format` set to `diarized_json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextSegmentEvent {
    /// The type of the event. Always `transcript.text.segment`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Unique identifier for the segment.
    pub id: String,
    /// Start timestamp of the segment in seconds.
    pub start: f64,
    /// End timestamp of the segment in seconds.
    pub end: f64,
    /// Transcript text for this segment.
    pub text: String,
    /// Speaker label for this segment.
    pub speaker: String,
}
