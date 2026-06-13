// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserUser` DTO.

use serde::{Deserialize, Serialize};

/// Nested user details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUser {
    pub object: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banned: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banned_at: Option<i64>,
}
