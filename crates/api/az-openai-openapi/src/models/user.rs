// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `User` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UserProjects,
    UserUser,
};

/// Represents an individual `user` within an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The object type, which is always `organization.user`
    pub object: String,
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// `owner` or `reader`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The Unix timestamp (in seconds) of when the user was added.
    pub added_at: i64,
    /// Whether this is the organization's default user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    /// The Unix timestamp (in seconds) of when the user was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    /// Nested user details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<UserUser>,
    /// Whether the user is a service account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_service_account: Option<bool>,
    /// Whether the user is an authorized purchaser for Scale Tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scale_tier_authorized_purchaser: Option<bool>,
    /// Whether the user is managed through SCIM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scim_managed: Option<bool>,
    /// The Unix timestamp (in seconds) of the user's last API key usage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_last_used_at: Option<i64>,
    /// The technical level metadata for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_level: Option<String>,
    /// The developer persona metadata for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_persona: Option<String>,
    /// Projects associated with the user, if included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projects: Option<UserProjects>,
}
