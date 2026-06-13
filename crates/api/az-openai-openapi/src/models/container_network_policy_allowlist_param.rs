// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerNetworkPolicyAllowlistParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerNetworkPolicyDomainSecretParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkPolicyAllowlistParam {
    /// Allow outbound network access only to specified domains. Always `allowlist`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A list of allowed domains when type is `allowlist`.
    pub allowed_domains: Vec<String>,
    /// Optional domain-scoped secrets for allowlisted domains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_secrets: Option<Vec<ContainerNetworkPolicyDomainSecretParam>>,
}
