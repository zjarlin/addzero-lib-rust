// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UpdateVectorStoreRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
    UpdateVectorStoreRequestExpiresAfter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVectorStoreRequest {
    /// The name of the vector store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<UpdateVectorStoreRequestExpiresAfter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
