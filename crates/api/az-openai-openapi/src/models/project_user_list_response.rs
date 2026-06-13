// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectUserListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectUser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUserListResponse {
    pub object: String,
    pub data: Vec<ProjectUser>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
