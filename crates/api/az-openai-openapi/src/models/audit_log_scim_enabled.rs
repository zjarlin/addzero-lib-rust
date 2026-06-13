// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogScimEnabled` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogScimEnabled {
    /// The ID of the SCIM was enabled for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
