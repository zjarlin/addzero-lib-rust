// Generated from OpenAPI spec. Do not edit by hand.
//! `SpeechAudioDoneEventUsage` DTO.

use serde::{Deserialize, Serialize};

/// Token usage statistics for the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAudioDoneEventUsage {
    /// Number of input tokens in the prompt.
    pub input_tokens: i32,
    /// Number of output tokens generated.
    pub output_tokens: i32,
    /// Total number of tokens used (input + output).
    pub total_tokens: i32,
}
