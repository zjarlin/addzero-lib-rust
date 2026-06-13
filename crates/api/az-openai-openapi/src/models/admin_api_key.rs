// Generated from OpenAPI spec. Do not edit by hand.
//! `AdminApiKey` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AdminApiKeyOwner,
};

/// Represents an individual Admin API key in an org.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiKey {
    /// The object type, which is always `organization.admin_api_key`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The redacted value of the API key
    pub redacted_value: String,
    /// The Unix timestamp (in seconds) of when the API key was created
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<i64>,
    pub owner: AdminApiKeyOwner,
}
