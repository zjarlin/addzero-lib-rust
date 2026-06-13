// Generated from OpenAPI spec. Do not edit by hand.
//! `UpdateVectorStoreFileAttributesRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileAttributes,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVectorStoreFileAttributesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<VectorStoreFileAttributes>,
}
