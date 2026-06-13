// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `DeleteFileResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
