// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogServiceAccountCreatedData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to create the service account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogServiceAccountCreatedData {
    /// The role of the service account. Is either `owner` or `member`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
