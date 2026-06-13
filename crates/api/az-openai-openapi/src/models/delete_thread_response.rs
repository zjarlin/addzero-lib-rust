// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `DeleteThreadResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteThreadResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}
