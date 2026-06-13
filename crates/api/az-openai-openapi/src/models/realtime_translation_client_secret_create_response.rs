// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationClientSecretCreateResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSession,
};

/// Response from creating a translation session and client secret for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationClientSecretCreateResponse {
    /// The generated client secret value.
    pub value: String,
    /// Expiration timestamp for the client secret, in seconds since epoch.
    pub expires_at: i64,
    pub session: RealtimeTranslationSession,
}
