// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunObjectIncompleteDetails` DTO.

use serde::{Deserialize, Serialize};

/// Details on why the run is incomplete. Will be `null` if the run is not incomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObjectIncompleteDetails {
    /// The reason why the run is incomplete. This will point to which specific token limit was reached over
    /// the course of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
