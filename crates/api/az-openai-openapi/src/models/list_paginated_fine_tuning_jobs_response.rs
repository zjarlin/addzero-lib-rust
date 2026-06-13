// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListPaginatedFineTuningJobsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningJob,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPaginatedFineTuningJobsResponse {
    pub data: Vec<FineTuningJob>,
    pub has_more: bool,
    pub object: String,
}
