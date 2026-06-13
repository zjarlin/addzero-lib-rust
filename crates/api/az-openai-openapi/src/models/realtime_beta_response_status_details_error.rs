// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaResponseStatusDetailsError` DTO.

use serde::{Deserialize, Serialize};

/// A description of the error that caused the response to fail, populated when the `status` is
/// `failed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaResponseStatusDetailsError {
    /// The type of error.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// Error code, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
