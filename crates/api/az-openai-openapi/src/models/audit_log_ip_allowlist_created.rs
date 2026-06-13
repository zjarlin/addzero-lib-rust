// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogIpAllowlistCreated` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistCreated {
    /// The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The IP addresses or CIDR ranges included in the configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
}
