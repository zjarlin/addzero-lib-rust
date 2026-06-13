// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectApiKeyListResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectApiKey,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApiKeyListResponse {
    pub object: String,
    pub data: Vec<ProjectApiKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
