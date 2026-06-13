// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileExpirationAfter` DTO.

use serde::{Deserialize, Serialize};

/// The expiration policy for a file. By default, files with `purpose=batch` expire after 30 days and
/// all other files are persisted until they are manually deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExpirationAfter {
    /// Anchor timestamp after which the expiration policy applies. Supported anchors: `created_at`.
    pub anchor: String,
    /// The number of seconds after the anchor time that the file will expire. Must be between 3600 (1 hour)
    /// and 2592000 (30 days).
    pub seconds: i64,
}
