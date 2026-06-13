// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogUserUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogUserUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogUserUpdated {
    /// The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogUserUpdatedChangesRequested>,
}
