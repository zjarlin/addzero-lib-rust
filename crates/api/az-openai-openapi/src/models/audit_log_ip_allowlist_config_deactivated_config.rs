// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogIpAllowlistConfigDeactivatedConfig` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistConfigDeactivatedConfig {
    /// The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
