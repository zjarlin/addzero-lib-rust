// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UsageTimeBucket` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UsageTimeBucketResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTimeBucket {
    pub object: String,
    pub start_time: i32,
    pub end_time: i32,
    pub results: Vec<UsageTimeBucketResult>,
}
