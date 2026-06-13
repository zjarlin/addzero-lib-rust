// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListFilesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OpenAIFile,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    pub object: String,
    pub data: Vec<OpenAIFile>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
