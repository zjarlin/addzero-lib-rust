// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptionDiarizedSegment` DTO.

use serde::{Deserialize, Serialize};

/// A segment of diarized transcript text with speaker metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionDiarizedSegment {
    /// The type of the segment. Always `transcript.text.segment`.
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
    /// Speaker label for this segment. When known speakers are provided, the label matches
    /// `known_speaker_names[]`. Otherwise speakers are labeled sequentially using capital letters (`A`,
    /// `B`, ...).
    pub speaker: String,
}
