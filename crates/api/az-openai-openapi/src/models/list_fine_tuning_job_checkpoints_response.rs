// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ListFineTuningJobCheckpointsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningJobCheckpoint,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFineTuningJobCheckpointsResponse {
    pub data: Vec<FineTuningJobCheckpoint>,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
