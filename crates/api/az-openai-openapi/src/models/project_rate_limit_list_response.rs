// Generated from OpenAPI spec. Do not edit by hand.
//! `ProjectRateLimitListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectRateLimit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRateLimitListResponse {
    pub object: String,
    pub data: Vec<ProjectRateLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
