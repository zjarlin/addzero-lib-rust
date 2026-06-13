// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ClosedStatus` DTO.

use serde::{Deserialize, Serialize};

/// Indicates that a thread has been closed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedStatus {
    /// Status discriminator that is always `closed`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
