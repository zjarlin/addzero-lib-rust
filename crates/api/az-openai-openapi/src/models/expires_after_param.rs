// Generated from OpenAPI spec. Do not edit by hand.
//! `ExpiresAfterParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls when the session expires relative to an anchor timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiresAfterParam {
    /// Base timestamp used to calculate expiration. Currently fixed to `created_at`.
    pub anchor: String,
    /// Number of seconds after the anchor when the session expires.
    pub seconds: i64,
}
