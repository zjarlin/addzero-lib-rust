// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateContainerBodyExpiresAfter` DTO.

use serde::{Deserialize, Serialize};

/// Container expiration time in seconds relative to the 'anchor' time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerBodyExpiresAfter {
    /// Time anchor for the expiration time. Currently only 'last_active_at' is supported.
    pub anchor: String,
    pub minutes: i32,
}
