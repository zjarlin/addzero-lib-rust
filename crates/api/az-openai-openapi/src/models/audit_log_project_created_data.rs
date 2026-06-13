// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogProjectCreatedData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to create the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProjectCreatedData {
    /// The project name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The title of the project as seen on the dashboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
