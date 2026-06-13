// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogProjectUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogProjectUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProjectUpdated {
    /// The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogProjectUpdatedChangesRequested>,
}
