// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCreateClientSecretRequestExpiresAfter` DTO.

use serde::{Deserialize, Serialize};

/// Configuration for the client secret expiration. Expiration refers to the time after which a client
/// secret will no longer be valid for creating sessions. The session itself may continue after that
/// time once started. A secret can be used to create multiple sessions until it expires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCreateClientSecretRequestExpiresAfter {
    /// The anchor point for the client secret expiration, meaning that `seconds` will be added to the
    /// `created_at` time of the client secret to produce an expiration timestamp. Only `created_at` is
    /// currently supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// The number of seconds from the anchor point to the expiration. Select a value between `10` and
    /// `7200` (2 hours). This default to 600 seconds (10 minutes) if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<i64>,
}
