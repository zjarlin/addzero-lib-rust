// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerResourceNetworkPolicy` DTO.

use serde::{Deserialize, Serialize};

/// Network access policy for the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResourceNetworkPolicy {
    /// The network policy mode.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Allowed outbound domains when `type` is `allowlist`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}
