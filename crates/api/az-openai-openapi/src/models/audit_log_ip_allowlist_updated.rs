// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogIpAllowlistUpdated` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistUpdated {
    /// The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The updated set of IP addresses or CIDR ranges in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
}
