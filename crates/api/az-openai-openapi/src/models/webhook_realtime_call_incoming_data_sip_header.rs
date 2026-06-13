// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookRealtimeCallIncomingDataSipHeader` DTO.

use serde::{Deserialize, Serialize};

/// A header from the SIP Invite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRealtimeCallIncomingDataSipHeader {
    /// Name of the SIP Header.
    pub name: String,
    /// Value of the SIP Header.
    pub value: String,
}
