// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateContainerBodyNetworkPolicy` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerNetworkPolicyAllowlistParam,
    ContainerNetworkPolicyDisabledParam,
};

/// Network access policy for the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateContainerBodyNetworkPolicy {
    ContainerNetworkPolicyDisabledParam(ContainerNetworkPolicyDisabledParam),
    ContainerNetworkPolicyAllowlistParam(ContainerNetworkPolicyAllowlistParam),
}
