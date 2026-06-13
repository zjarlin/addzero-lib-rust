// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseGAAudioOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeAudioFormats,
    VoiceIdsShared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseGAAudioOutput {
    /// The format of the output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    /// The voice the model uses to respond. Voice cannot be changed during the session once the model has
    /// responded with audio at least once. Current voice options are `alloy`, `ash`, `ballad`, `coral`,
    /// `echo`, `sage`, `shimmer`, `verse`, `marin`, and `cedar`. We recommend `marin` and `cedar` for best
    /// quality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceIdsShared>,
    /// The speed of the model's spoken response as a multiple of the original speed. 1.0 is the default
    /// speed. 0.25 is the minimum speed. 1.5 is the maximum speed. This value can only be changed in
    /// between model turns, not while a response is in progress. This parameter is a post-processing
    /// adjustment to the audio after it is generated, it's also possible to prompt the model to speak
    /// faster or slower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}
