// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptionSegment` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Unique identifier of the segment.
    pub id: i32,
    /// Seek offset of the segment.
    pub seek: i32,
    /// Start time of the segment in seconds.
    pub start: f64,
    /// End time of the segment in seconds.
    pub end: f64,
    /// Text content of the segment.
    pub text: String,
    /// Array of token IDs for the text content.
    pub tokens: Vec<i32>,
    /// Temperature parameter used for generating the segment.
    pub temperature: f32,
    /// Average logprob of the segment. If the value is lower than -1, consider the logprobs failed.
    pub avg_logprob: f32,
    /// Compression ratio of the segment. If the value is greater than 2.4, consider the compression failed.
    pub compression_ratio: f32,
    /// Probability of no speech in the segment. If the value is higher than 1.0 and the `avg_logprob` is
    /// below -1, consider this segment silent.
    pub no_speech_prob: f32,
}
