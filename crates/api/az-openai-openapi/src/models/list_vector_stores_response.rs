// Generated from OpenAPI spec. Do not edit by hand.
//! `ListVectorStoresResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    VectorStoreObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVectorStoresResponse {
    pub object: String,
    pub data: Vec<VectorStoreObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
