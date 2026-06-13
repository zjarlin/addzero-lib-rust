// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogGroupDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogGroupDeleted {
    /// The ID of the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
