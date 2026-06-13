// Generated from OpenAPI spec. Do not edit by hand.
//! `AutoCodeInterpreterToolParamNetworkPolicy` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerNetworkPolicyAllowlistParam,
    ContainerNetworkPolicyDisabledParam,
};

/// Network access policy for the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AutoCodeInterpreterToolParamNetworkPolicy {
    ContainerNetworkPolicyDisabledParam(ContainerNetworkPolicyDisabledParam),
    ContainerNetworkPolicyAllowlistParam(ContainerNetworkPolicyAllowlistParam),
}
