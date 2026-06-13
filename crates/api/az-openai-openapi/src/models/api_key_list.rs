// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ApiKeyList` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AdminApiKey,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyList {
    pub object: String,
    pub data: Vec<AdminApiKey>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}
