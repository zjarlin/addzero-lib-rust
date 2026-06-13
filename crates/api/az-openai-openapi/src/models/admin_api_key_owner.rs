// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AdminApiKeyOwner` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiKeyOwner {
    /// Always `user`
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The object type, which is always organization.user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The identifier, which can be referenced in API endpoints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Unix timestamp (in seconds) of when the user was created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Always `owner`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
