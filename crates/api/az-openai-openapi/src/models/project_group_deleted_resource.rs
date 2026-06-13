// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectGroupDeletedResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after removing a group from a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGroupDeletedResource {
    /// Always `project.group.deleted`.
    pub object: String,
    /// Whether the group membership in the project was removed.
    pub deleted: bool,
}
