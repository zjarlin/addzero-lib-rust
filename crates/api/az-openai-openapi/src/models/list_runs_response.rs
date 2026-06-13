// Generated from OpenAPI spec. Do not edit by hand.
//! `ListRunsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRunsResponse {
    pub object: String,
    pub data: Vec<RunObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
