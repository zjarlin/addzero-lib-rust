// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogRateLimitUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogRateLimitUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRateLimitUpdated {
    /// The rate limit ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the rate limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogRateLimitUpdatedChangesRequested>,
}
