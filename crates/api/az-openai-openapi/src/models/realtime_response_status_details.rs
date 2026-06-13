// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeResponseStatusDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeResponseStatusDetailsError,
};

/// Additional details about the status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeResponseStatusDetails {
    /// The type of error that caused the response to fail, corresponding with the `status` field
    /// (`completed`, `cancelled`, `incomplete`, `failed`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    /// The reason the Response did not complete. For a `cancelled` Response, one of `turn_detected` (the
    /// server VAD detected a new start of speech) or `client_cancelled` (the client sent a cancel event).
    /// For an `incomplete` Response, one of `max_output_tokens` or `content_filter` (the server-side safety
    /// filter activated and cut off the response).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// A description of the error that caused the response to fail, populated when the `status` is
    /// `failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RealtimeResponseStatusDetailsError>,
}
