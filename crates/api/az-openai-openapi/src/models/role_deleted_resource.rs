// Generated from OpenAPI spec. Do not edit by hand.
//! `RoleDeletedResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after deleting a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDeletedResource {
    /// Always `role.deleted`.
    pub object: String,
    /// Identifier of the deleted role.
    pub id: String,
    /// Whether the role was deleted.
    pub deleted: bool,
}
