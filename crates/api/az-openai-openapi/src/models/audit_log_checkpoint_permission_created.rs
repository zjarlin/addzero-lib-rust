// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogCheckpointPermissionCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogCheckpointPermissionCreatedData,
};

/// The project and fine-tuned model checkpoint that the checkpoint permission was created for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCheckpointPermissionCreated {
    /// The ID of the checkpoint permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to create the checkpoint permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogCheckpointPermissionCreatedData>,
}
