// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogIpAllowlistDeleted` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistDeleted {
    /// The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The IP addresses or CIDR ranges that were in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
}
