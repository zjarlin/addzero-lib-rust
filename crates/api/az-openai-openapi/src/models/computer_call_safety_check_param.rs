// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerCallSafetyCheckParam` DTO.

use serde::{Deserialize, Serialize};

/// A pending safety check for the computer call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerCallSafetyCheckParam {
    /// The ID of the pending safety check.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
