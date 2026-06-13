// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogRoleAssignmentCreated` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRoleAssignmentCreated {
    /// The identifier of the role assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The principal (user or group) that received the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// The type of principal (user or group) that received the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,
    /// The resource the role assignment is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// The type of resource the role assignment is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
}
