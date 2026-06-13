// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `LockedStatus` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that a thread is locked and cannot accept new input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedStatus {
    /// Status discriminator that is always `locked`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
