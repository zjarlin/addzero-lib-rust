// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserRoleUpdateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleUpdateRequest {
    /// `owner` or `reader`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Role ID to assign to the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    /// Technical level metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_level: Option<String>,
    /// Developer persona metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_persona: Option<String>,
}
