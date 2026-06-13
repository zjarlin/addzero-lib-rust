// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeCallReferRequest` DTO.

use serde::{Deserialize, Serialize};

/// Parameters required to transfer a SIP call to a new destination using the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeCallReferRequest {
    /// URI that should appear in the SIP Refer-To header. Supports values like `tel:+14155550123` or
    /// `sip:agent@example.com`.
    pub target_uri: String,
}
