// Generated from OpenAPI spec. Do not edit by hand.
//! `DeleteVectorStoreFileResponse` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVectorStoreFileResponse {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}
