// Generated from OpenAPI spec. Do not edit by hand.
//! `BatchFileExpirationAfter` DTO.

use serde::{Deserialize, Serialize};

/// The expiration policy for the output and/or error file that are generated for a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFileExpirationAfter {
    /// Anchor timestamp after which the expiration policy applies. Supported anchors: `created_at`. Note
    /// that the anchor is the file creation time, not the time the batch is created.
    pub anchor: String,
    /// The number of seconds after the anchor time that the file will expire. Must be between 3600 (1 hour)
    /// and 2592000 (30 days).
    pub seconds: i64,
}
