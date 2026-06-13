// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectApiKeyOwnerServiceAccount` DTO.

use serde::{Deserialize, Serialize};

/// The service account that owns a project API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKeyOwnerServiceAccount {
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the service account.
    pub name: String,
    /// The Unix timestamp (in seconds) of when the service account was created.
    pub created_at: i64,
    /// The service account's project role.
    pub role: String,
}
