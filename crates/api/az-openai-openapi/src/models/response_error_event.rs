// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseErrorEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when an error occurs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseErrorEvent {
    /// The type of the event. Always `error`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// The error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
