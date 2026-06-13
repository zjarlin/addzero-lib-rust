// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerNetworkPolicyDomainSecretParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkPolicyDomainSecretParam {
    /// The domain associated with the secret.
    pub domain: String,
    /// The name of the secret to inject for the domain.
    pub name: String,
    /// The secret value to inject for the domain.
    pub value: String,
}
