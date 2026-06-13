// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectApiKeyOwnerUser` DTO.

use serde::{Deserialize, Serialize};

/// The user that owns a project API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKeyOwnerUser {
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The email address of the user.
    pub email: String,
    /// The name of the user.
    pub name: String,
    /// The Unix timestamp (in seconds) of when the user was created.
    pub created_at: i64,
    /// The user's project role.
    pub role: String,
}
