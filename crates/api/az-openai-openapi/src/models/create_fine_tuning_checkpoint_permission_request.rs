// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateFineTuningCheckpointPermissionRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningCheckpointPermissionRequest {
    /// The project identifiers to grant access to.
    pub project_ids: Vec<String>,
}
