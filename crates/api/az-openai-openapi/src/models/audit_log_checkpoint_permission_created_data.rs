// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogCheckpointPermissionCreatedData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to create the checkpoint permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCheckpointPermissionCreatedData {
    /// The ID of the project that the checkpoint permission was created for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// The ID of the fine-tuned model checkpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_tuned_model_checkpoint: Option<String>,
}
