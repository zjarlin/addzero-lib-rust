// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectUserUpdateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUserUpdateRequest {
    /// `owner` or `member`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
