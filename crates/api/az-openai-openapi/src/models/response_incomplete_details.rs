// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseIncompleteDetails` DTO.

use serde::{Deserialize, Serialize};

/// Details about why the response is incomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseIncompleteDetails {
    /// The reason why the response is incomplete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
