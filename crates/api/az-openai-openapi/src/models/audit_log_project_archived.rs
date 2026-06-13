// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogProjectArchived` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogProjectArchived {
    /// The project ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
