// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `DeleteMessageResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMessageResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}
