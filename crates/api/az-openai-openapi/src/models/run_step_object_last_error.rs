// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepObjectLastError` DTO.

use serde::{Deserialize, Serialize};

/// The last error associated with this run step. Will be `null` if there are no errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepObjectLastError {
    /// One of `server_error` or `rate_limit_exceeded`.
    pub code: String,
    /// A human-readable description of the error.
    pub message: String,
}
