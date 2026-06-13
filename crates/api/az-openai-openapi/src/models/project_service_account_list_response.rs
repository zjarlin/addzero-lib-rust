// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectServiceAccountListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectServiceAccount,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccountListResponse {
    pub object: String,
    pub data: Vec<ProjectServiceAccount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
