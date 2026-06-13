// Generated from OpenAPI spec. Do not edit by hand.
//! `ListBatchesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Batch,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBatchesResponse {
    pub data: Vec<Batch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
    pub object: String,
}
