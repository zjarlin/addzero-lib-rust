// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalApiError` DTO.

use serde::{Deserialize, Serialize};

/// An object representing an error response from the Eval API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalApiError {
    /// The error code.
    pub code: String,
    /// The error message.
    pub message: String,
}
