// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogProject` DTO.

use serde::{Deserialize, Serialize};

/// The project that the action was scoped to. Absent for actions not scoped to projects. Note that any
/// admin actions taken via Admin API keys are associated with the default project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProject {
    /// The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The project title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
