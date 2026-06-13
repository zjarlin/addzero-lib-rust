// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogInviteSentData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to create the invite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogInviteSentData {
    /// The email invited to the organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The role the email was invited to be. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
