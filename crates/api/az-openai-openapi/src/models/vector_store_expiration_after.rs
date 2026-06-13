// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreExpirationAfter` DTO.

use serde::{Deserialize, Serialize};

/// The expiration policy for a vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreExpirationAfter {
    /// Anchor timestamp after which the expiration policy applies. Supported anchors: `last_active_at`.
    pub anchor: String,
    /// The number of days after the anchor time that the vector store will expire.
    pub days: i32,
}
