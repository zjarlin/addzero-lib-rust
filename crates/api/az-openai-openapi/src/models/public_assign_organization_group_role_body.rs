// Generated from OpenAPI spec. Do not edit by hand.
//! `PublicAssignOrganizationGroupRoleBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for assigning a role to a group or user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAssignOrganizationGroupRoleBody {
    /// Identifier of the role to assign.
    pub role_id: String,
}
