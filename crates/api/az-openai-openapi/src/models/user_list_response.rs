// Generated from OpenAPI spec. Do not edit by hand.
//! `UserListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    User,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListResponse {
    pub object: String,
    pub data: Vec<User>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
