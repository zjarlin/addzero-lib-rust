// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogRoleUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogRoleUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRoleUpdated {
    /// The role ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogRoleUpdatedChangesRequested>,
}
