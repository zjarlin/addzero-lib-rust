// Generated from OpenAPI spec. Do not edit by hand.
//! `UpdateVoiceConsentRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVoiceConsentRequest {
    /// The updated label for this consent recording.
    pub name: String,
}
