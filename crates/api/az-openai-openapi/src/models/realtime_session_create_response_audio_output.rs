// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseAudioOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeAudioFormats,
    VoiceIdsShared,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateResponseAudioOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RealtimeAudioFormats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceIdsShared>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}
