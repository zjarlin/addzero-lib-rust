// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptTextDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextDeltaEventLogprob,
};

/// Emitted when there is an additional text delta. This is also the first event emitted when the
/// transcription starts. Only emitted when you [create a transcription](/docs/api-
/// reference/audio/create-transcription) with the `Stream` parameter set to `true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextDeltaEvent {
    /// The type of the event. Always `transcript.text.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text delta that was additionally transcribed.
    pub delta: String,
    /// The log probabilities of the delta. Only included if you [create a transcription](/docs/api-
    /// reference/audio/create-transcription) with the `include[]` parameter set to `logprobs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<TranscriptTextDeltaEventLogprob>>,
    /// Identifier of the diarized segment that this delta belongs to. Only present when using
    /// `gpt-4o-transcribe-diarize`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
}
