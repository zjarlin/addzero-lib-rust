// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectUserCreateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUserCreateRequest {
    /// The ID of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Email of the user to add.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// `owner` or `member`
    pub role: String,
}
