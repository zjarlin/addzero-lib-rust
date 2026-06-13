// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateTranscriptionResponseJson` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateTranscriptionResponseJsonLogprob,
    CreateTranscriptionResponseJsonUsage,
};

/// Represents a transcription response returned by model, based on the provided input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTranscriptionResponseJson {
    /// The transcribed text.
    pub text: String,
    /// The log probabilities of the tokens in the transcription. Only returned with the models
    /// `gpt-4o-transcribe` and `gpt-4o-mini-transcribe` if `logprobs` is added to the `include` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<CreateTranscriptionResponseJsonLogprob>>,
    /// Token usage statistics for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<CreateTranscriptionResponseJsonUsage>,
}
