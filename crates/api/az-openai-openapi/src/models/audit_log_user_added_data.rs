// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogUserAddedData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to add the user to the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogUserAddedData {
    /// The role of the user. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
