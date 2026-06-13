// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationClientSecretCreateRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationClientSecretCreateRequestExpiresAfter,
    RealtimeTranslationSessionCreateRequest,
};

/// Create a translation session and client secret for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationClientSecretCreateRequest {
    /// Configuration for the client secret expiration. Expiration refers to the time after which a client
    /// secret will no longer be valid for creating sessions. The session itself may continue after that
    /// time once started. A secret can be used to create multiple sessions until it expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<RealtimeTranslationClientSecretCreateRequestExpiresAfter>,
    pub session: RealtimeTranslationSessionCreateRequest,
}
