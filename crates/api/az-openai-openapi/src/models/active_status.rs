// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ActiveStatus` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that a thread is active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatus {
    /// Status discriminator that is always `active`.
    #[serde(rename = "type")]
    pub type_value: String,
}
