// Generated from OpenAPI spec. Do not edit by hand.
//! `VoiceIdsOrCustomVoiceObject` DTO.

use serde::{Deserialize, Serialize};

/// Custom voice reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceIdsOrCustomVoiceObject {
    /// The custom voice ID, e.g. `voice_1234`.
    pub id: String,
}
