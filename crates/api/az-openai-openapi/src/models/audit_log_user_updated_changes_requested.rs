// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogUserUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to update the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogUserUpdatedChangesRequested {
    /// The role of the user. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
