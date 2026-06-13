// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `DeleteFineTuningCheckpointPermissionResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFineTuningCheckpointPermissionResponse {
    /// The ID of the fine-tuned model checkpoint permission that was deleted.
    pub id: String,
    /// The object type, which is always "checkpoint.permission".
    pub object: String,
    /// Whether the fine-tuned model checkpoint permission was successfully deleted.
    pub deleted: bool,
}
