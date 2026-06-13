// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuningJobError` DTO.

use serde::{Deserialize, Serialize};

/// For fine-tuning jobs that have `failed`, this will contain more information on the cause of the
/// failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobError {
    /// A machine-readable error code.
    pub code: String,
    /// A human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}
