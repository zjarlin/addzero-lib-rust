// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectUser` DTO.

use serde::{Deserialize, Serialize};

/// Represents an individual user in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUser {
    /// The object type, which is always `organization.project.user`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// `owner` or `member`
    pub role: String,
    /// The Unix timestamp (in seconds) of when the project was added.
    pub added_at: i64,
}
