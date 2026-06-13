// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `TranscriptTextDoneEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    TranscriptTextDoneEventLogprob,
    TranscriptTextUsageTokens,
};

/// Emitted when the transcription is complete. Contains the complete transcription text. Only emitted
/// when you [create a transcription](/docs/api-reference/audio/create-transcription) with the `Stream`
/// parameter set to `true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextDoneEvent {
    /// The type of the event. Always `transcript.text.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text that was transcribed.
    pub text: String,
    /// The log probabilities of the individual tokens in the transcription. Only included if you [create a
    /// transcription](/docs/api-reference/audio/create-transcription) with the `include[]` parameter set to
    /// `logprobs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<TranscriptTextDoneEventLogprob>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<TranscriptTextUsageTokens>,
}
