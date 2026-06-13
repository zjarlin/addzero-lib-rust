// Generated from OpenAPI spec. Do not edit by hand.
//! `VoiceIdsShared` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VoiceIdsShared {
    String(String),
    String2(String),
}
