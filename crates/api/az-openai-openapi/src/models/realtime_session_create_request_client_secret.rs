// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateRequestClientSecret` DTO.

use serde::{Deserialize, Serialize};

/// Ephemeral key returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeSessionCreateRequestClientSecret {
    /// Ephemeral key usable in client environments to authenticate connections to the Realtime API. Use
    /// this in client-side environments rather than a standard API token, which should only be used server-
    /// side.
    pub value: String,
    /// Timestamp for when the token expires. Currently, all tokens expire after one minute.
    pub expires_at: i64,
}
