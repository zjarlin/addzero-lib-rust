// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCallRejectRequest` DTO.

use serde::{Deserialize, Serialize};

/// Parameters used to decline an incoming SIP call handled by the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCallRejectRequest {
    /// SIP response code to send back to the caller. Defaults to `603` (Decline) when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i32>,
}
