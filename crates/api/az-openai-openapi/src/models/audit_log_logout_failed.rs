// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogLogoutFailed` DTO.

use serde::{Deserialize, Serialize};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogLogoutFailed {
    /// The error code of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// The error message of the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
