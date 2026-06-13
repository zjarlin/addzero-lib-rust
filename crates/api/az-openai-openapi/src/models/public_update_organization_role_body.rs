// Generated from OpenAPI spec. Do not edit by hand.
//! `PublicUpdateOrganizationRoleBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for updating an existing role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUpdateOrganizationRoleBody {
    /// Updated set of permissions for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    /// New description for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New name for the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
}
