// Generated from OpenAPI spec. Do not edit by hand.
//! `UsageResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UsageTimeBucket,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub object: String,
    pub data: Vec<UsageTimeBucket>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
}
