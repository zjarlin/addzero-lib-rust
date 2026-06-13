// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogOrganizationUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogOrganizationUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogOrganizationUpdated {
    /// The organization ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the organization settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogOrganizationUpdatedChangesRequested>,
}
