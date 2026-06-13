// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogRoleCreated` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRoleCreated {
    /// The role ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
    /// The permissions granted by the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    /// The type of resource the role belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// The resource the role is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}
