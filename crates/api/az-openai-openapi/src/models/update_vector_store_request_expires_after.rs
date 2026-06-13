// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UpdateVectorStoreRequestExpiresAfter` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVectorStoreRequestExpiresAfter {
    /// Anchor timestamp after which the expiration policy applies. Supported anchors: `last_active_at`.
    pub anchor: String,
    /// The number of days after the anchor time that the vector store will expire.
    pub days: i32,
}
