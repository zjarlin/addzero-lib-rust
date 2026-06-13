// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogRateLimitDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRateLimitDeleted {
    /// The rate limit ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
