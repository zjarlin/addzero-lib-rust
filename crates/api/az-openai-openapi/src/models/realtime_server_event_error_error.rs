// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventErrorError` DTO.

use serde::{Deserialize, Serialize};

/// Details of the error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventErrorError {
    /// The type of error (e.g., "invalid_request_error", "server_error").
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// A human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}
