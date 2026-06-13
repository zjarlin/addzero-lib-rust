// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectServiceAccountApiKey` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccountApiKey {
    /// The object type, which is always `organization.project.service_account.api_key`
    pub object: String,
    pub value: String,
    pub name: String,
    pub created_at: i64,
    pub id: String,
}
