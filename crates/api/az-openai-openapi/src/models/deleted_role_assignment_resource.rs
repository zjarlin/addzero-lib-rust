// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedRoleAssignmentResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after unassigning a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedRoleAssignmentResource {
    /// Identifier for the deleted assignment, such as `group.role.deleted` or `user.role.deleted`.
    pub object: String,
    /// Whether the assignment was removed.
    pub deleted: bool,
}
