// Generated from OpenAPI spec. Do not edit by hand.
//! `ListFineTuningCheckpointPermissionResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningCheckpointPermission,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFineTuningCheckpointPermissionResponse {
    pub data: Vec<FineTuningCheckpointPermission>,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    pub has_more: bool,
}
