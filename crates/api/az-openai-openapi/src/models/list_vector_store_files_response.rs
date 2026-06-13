// Generated from OpenAPI spec. Do not edit by hand.
//! `ListVectorStoreFilesResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreFileObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVectorStoreFilesResponse {
    pub object: String,
    pub data: Vec<VectorStoreFileObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
