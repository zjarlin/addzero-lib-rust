// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectApiKey` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectApiKeyOwner,
};

/// Represents an individual API key in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKey {
    /// The object type, which is always `organization.project.api_key`
    pub object: String,
    /// The redacted value of the API key
    pub redacted_value: String,
    /// The name of the API key
    pub name: String,
    /// The Unix timestamp (in seconds) of when the API key was created
    pub created_at: i64,
    /// The Unix timestamp (in seconds) of when the API key was last used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<i64>,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    pub owner: ProjectApiKeyOwner,
}
