// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionRequestMessageContentPartAudioInputAudio,
};

/// Learn about [audio inputs](/docs/guides/audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartAudio {
    /// The type of the content part. Always `input_audio`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub input_audio: ChatCompletionRequestMessageContentPartAudioInputAudio,
}
