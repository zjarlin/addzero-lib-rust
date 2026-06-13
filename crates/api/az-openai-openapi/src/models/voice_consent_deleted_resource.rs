// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VoiceConsentDeletedResource` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConsentDeletedResource {
    /// The consent recording identifier.
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
