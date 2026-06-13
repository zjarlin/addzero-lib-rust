// Generated from OpenAPI spec. Do not edit by hand.
//! `ContainerReferenceResource` DTO.

use serde::{Deserialize, Serialize};

/// Represents a container created with /v1/containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerReferenceResource {
    /// The environment type. Always `container_reference`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub container_id: String,
}
