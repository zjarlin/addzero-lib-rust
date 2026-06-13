// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerResource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerListResource {
    /// The type of object returned, must be 'list'.
    pub object: String,
    /// A list of containers.
    pub data: Vec<ContainerResource>,
    /// The ID of the first container in the list.
    pub first_id: String,
    /// The ID of the last container in the list.
    pub last_id: String,
    /// Whether there are more containers available.
    pub has_more: bool,
}
