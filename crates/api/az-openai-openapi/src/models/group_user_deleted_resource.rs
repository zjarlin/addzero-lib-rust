// Generated from OpenAPI spec. Do not edit by hand.
//! `GroupUserDeletedResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after removing a user from a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUserDeletedResource {
    /// Always `group.user.deleted`.
    pub object: String,
    /// Whether the group membership was removed.
    pub deleted: bool,
}
