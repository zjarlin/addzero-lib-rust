// Generated from OpenAPI spec. Do not edit by hand.
//! `DeleteModelResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteModelResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}
