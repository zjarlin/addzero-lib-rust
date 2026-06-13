// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogActorUser` DTO.

use serde::{Deserialize, Serialize};

/// The user who performed the audit logged action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogActorUser {
    /// The user id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The user email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
