// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectGroup` DTO.

use serde::{Deserialize, Serialize};

/// Details about a group's membership in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGroup {
    /// Always `project.group`.
    pub object: String,
    /// Identifier of the project.
    pub project_id: String,
    /// Identifier of the group that has access to the project.
    pub group_id: String,
    /// Display name of the group.
    pub group_name: String,
    /// The type of the group.
    pub group_type: String,
    /// Unix timestamp (in seconds) when the group was granted project access.
    pub created_at: i64,
}
