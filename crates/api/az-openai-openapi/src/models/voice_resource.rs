// Generated from OpenAPI spec. Do not edit by hand.
//! `VoiceResource` DTO.

use serde::{Deserialize, Serialize};

/// A custom voice that can be used for audio output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceResource {
    /// The object type, which is always `audio.voice`.
    pub object: String,
    /// The voice identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The name of the voice.
    pub name: String,
    /// The Unix timestamp (in seconds) for when the voice was created.
    pub created_at: i64,
}
