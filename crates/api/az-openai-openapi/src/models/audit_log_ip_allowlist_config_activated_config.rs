// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogIpAllowlistConfigActivatedConfig` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogIpAllowlistConfigActivatedConfig {
    /// The ID of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the IP allowlist configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
