// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UserDeleteResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeleteResponse {
    pub object: String,
    pub id: String,
    pub deleted: bool,
}
