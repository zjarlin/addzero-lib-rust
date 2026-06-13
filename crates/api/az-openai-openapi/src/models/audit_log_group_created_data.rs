// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogGroupCreatedData` DTO.

use serde::{Deserialize, Serialize};

/// Information about the created group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogGroupCreatedData {
    /// The group name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
}
