// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectServiceAccount` DTO.

use serde::{Deserialize, Serialize};

/// Represents an individual service account in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccount {
    /// The object type, which is always `organization.project.service_account`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the service account
    pub name: String,
    /// `owner` or `member`
    pub role: String,
    /// The Unix timestamp (in seconds) of when the service account was created
    pub created_at: i64,
}
