// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogInviteSent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogInviteSentData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogInviteSent {
    /// The ID of the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to create the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogInviteSentData>,
}
