// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogRoleAssignmentDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRoleAssignmentDeleted {
    /// The identifier of the role assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The principal (user or group) that had the role removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// The type of principal (user or group) that had the role removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,
    /// The resource the role assignment was scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// The type of resource the role assignment was scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
}
