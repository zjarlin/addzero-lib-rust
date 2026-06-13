// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookRealtimeCallIncomingData` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookRealtimeCallIncomingDataSipHeader,
};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRealtimeCallIncomingData {
    /// The unique ID of this call.
    pub call_id: String,
    /// Headers from the SIP Invite.
    pub sip_headers: Vec<WebhookRealtimeCallIncomingDataSipHeader>,
}
