// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateChatCompletionRequestAudio` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VoiceIdsOrCustomVoice,
};

/// Parameters for audio output. Required when audio output is requested with `modalities: ["audio"]`.
/// [Learn more](/docs/guides/audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatCompletionRequestAudio {
    /// The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`,
    /// `coral`, `echo`, `fable`, `nova`, `onyx`, `sage`, `shimmer`, `marin`, and `cedar`. You may also
    /// provide a custom voice object with an `id`, for example `{ "id": "voice_1234" }`.
    pub voice: VoiceIdsOrCustomVoice,
    /// Specifies the output audio format. Must be one of `wav`, `mp3`, `flac`, `opus`, or `pcm16`.
    pub format: String,
}
