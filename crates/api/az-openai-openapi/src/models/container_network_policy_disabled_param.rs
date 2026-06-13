// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerNetworkPolicyDisabledParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkPolicyDisabledParam {
    /// Disable outbound network access. Always `disabled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
