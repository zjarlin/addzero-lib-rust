// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeResponseAudioOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeAudioFormats,
    VoiceIdsShared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseAudioOutput {
    /// The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// The voice the model uses to respond. Voice cannot be changed during the session once the model has
    /// responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`,
    /// `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. We recommend `marin` and `cedar` for best
    /// quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceIdsShared>,
}
