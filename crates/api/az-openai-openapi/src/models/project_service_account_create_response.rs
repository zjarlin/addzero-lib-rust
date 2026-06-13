// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectServiceAccountCreateResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ProjectServiceAccountApiKey,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccountCreateResponse {
    pub object: String,
    pub id: String,
    pub name: String,
    /// Service accounts can only have one role of type `member`
    pub role: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ProjectServiceAccountApiKey>,
}
