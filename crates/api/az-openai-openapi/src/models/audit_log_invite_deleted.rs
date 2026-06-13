// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogInviteDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogInviteDeleted {
    /// The ID of the invite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
