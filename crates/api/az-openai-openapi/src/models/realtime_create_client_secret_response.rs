// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeCreateClientSecretResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeCreateClientSecretResponseSession,
};

/// Response from creating a session and client secret for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCreateClientSecretResponse {
    /// The generated client secret value.
    pub value: String,
    /// Expiration timestamp for the client secret, in seconds since epoch.
    pub expires_at: i64,
    /// The session configuration for either a realtime or transcription session.
    pub session: RealtimeCreateClientSecretResponseSession,
}
