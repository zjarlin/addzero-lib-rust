// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VoiceConsentResource` DTO.

use serde::{Deserialize, Serialize};

/// A consent recording used to authorize creation of a custom voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConsentResource {
    /// The object type, which is always `audio.voice_consent`.
    pub object: String,
    /// The consent recording identifier.
    pub id: String,
    /// The label provided when the consent recording was uploaded.
    pub name: String,
    /// The BCP 47 language tag for the consent phrase (for example, `en-US`).
    pub language: String,
    /// The Unix timestamp (in seconds) for when the consent recording was created.
    pub created_at: i64,
}
