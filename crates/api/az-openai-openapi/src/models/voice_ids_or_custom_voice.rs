// Generated from OpenAPI spec. Do not edit by hand.
//! `VoiceIdsOrCustomVoice` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VoiceIdsOrCustomVoiceObject,
    VoiceIdsShared,
};

/// A built-in voice name or a custom voice reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VoiceIdsOrCustomVoice {
    VoiceIdsShared(VoiceIdsShared),
    Object(VoiceIdsOrCustomVoiceObject),
}
