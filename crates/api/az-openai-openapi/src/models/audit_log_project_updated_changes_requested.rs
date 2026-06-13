// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogProjectUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to update the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProjectUpdatedChangesRequested {
    /// The title of the project as seen on the dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
