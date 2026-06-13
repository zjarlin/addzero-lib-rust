// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ContainerFileListResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ContainerFileResource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerFileListResource {
    /// The type of object returned, must be 'list'.
    pub object: String,
    /// A list of container files.
    pub data: Vec<ContainerFileResource>,
    /// The ID of the first file in the list.
    pub first_id: String,
    /// The ID of the last file in the list.
    pub last_id: String,
    /// Whether there are more files available.
    pub has_more: bool,
}
