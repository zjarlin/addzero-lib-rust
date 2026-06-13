// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerReferenceParam` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerReferenceParam {
    /// References a container created with the /v1/containers endpoint
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the referenced container.
    pub container_id: String,
}
