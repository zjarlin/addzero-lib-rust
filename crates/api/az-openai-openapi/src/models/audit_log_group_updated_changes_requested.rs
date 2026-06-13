// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogGroupUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to update the group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogGroupUpdatedChangesRequested {
    /// The updated group name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
}
