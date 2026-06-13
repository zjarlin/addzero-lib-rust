// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeResponseCreateParamsAudioOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeAudioFormats,
    VoiceIdsOrCustomVoice,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseCreateParamsAudioOutput {
    /// The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// The voice the model uses to respond. Supported built-in voices are `alloy`, `ash`, `ballad`,
    /// `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. You may also provide a custom
    /// voice object with an `id`, for example `{ "id": "voice_1234" }`. Voice cannot be changed during the
    /// session once the model has responded with audio at least once. We recommend `marin` and `cedar` for
    /// best quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceIdsOrCustomVoice>,
}
