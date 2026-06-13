// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogServiceAccountUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogServiceAccountUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogServiceAccountUpdated {
    /// The service account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to updated the service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogServiceAccountUpdatedChangesRequested>,
}
