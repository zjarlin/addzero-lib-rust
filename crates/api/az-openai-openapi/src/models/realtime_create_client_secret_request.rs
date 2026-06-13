// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCreateClientSecretRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeCreateClientSecretRequestExpiresAfter,
    RealtimeCreateClientSecretRequestSession,
};

/// Create a session and client secret for the Realtime API. The request can specify either a realtime
/// or a transcription session configuration. [Learn more about the Realtime
/// API](/docs/guides/realtime).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCreateClientSecretRequest {
    /// Configuration for the client secret expiration. Expiration refers to the time after which a client
    /// secret will no longer be valid for creating sessions. The session itself may continue after that
    /// time once started. A secret can be used to create multiple sessions until it expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<RealtimeCreateClientSecretRequestExpiresAfter>,
    /// Session configuration to use for the client secret. Choose either a realtime session or a
    /// transcription session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<RealtimeCreateClientSecretRequestSession>,
}
