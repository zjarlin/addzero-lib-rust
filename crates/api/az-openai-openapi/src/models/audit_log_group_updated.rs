// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogGroupUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogGroupUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogGroupUpdated {
    /// The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogGroupUpdatedChangesRequested>,
}
