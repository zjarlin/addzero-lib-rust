// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuningCheckpointPermission` DTO.

use serde::{Deserialize, Serialize};

/// The `checkpoint.permission` object represents a permission for a fine-tuned model checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningCheckpointPermission {
    /// The permission identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the permission was created.
    pub created_at: i64,
    /// The project identifier that the permission is for.
    pub project_id: String,
    /// The object type, which is always "checkpoint.permission".
    pub object: String,
}
