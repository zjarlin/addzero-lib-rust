// Generated from OpenAPI spec. Do not edit by hand.
//! `Error2` DTO.

use serde::{Deserialize, Serialize};

/// An error that occurred while generating the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error2 {
    /// A machine-readable error code that was returned.
    pub code: String,
    /// A human-readable description of the error that was returned.
    pub message: String,
}
