// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunObjectLastError` DTO.

use serde::{Deserialize, Serialize};

/// The last error associated with this run. Will be `null` if there are no errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObjectLastError {
    /// One of `server_error`, `rate_limit_exceeded`, or `invalid_prompt`.
    pub code: String,
    /// A human-readable description of the error.
    pub message: String,
}
