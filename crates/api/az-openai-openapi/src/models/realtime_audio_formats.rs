// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeAudioFormats` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeAudioFormatsPCMAAudioFormat,
    RealtimeAudioFormatsPCMAudioFormat,
    RealtimeAudioFormatsPCMUAudioFormat,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeAudioFormats {
    PCMAudioFormat(RealtimeAudioFormatsPCMAudioFormat),
    PCMUAudioFormat(RealtimeAudioFormatsPCMUAudioFormat),
    PCMAAudioFormat(RealtimeAudioFormatsPCMAAudioFormat),
}
