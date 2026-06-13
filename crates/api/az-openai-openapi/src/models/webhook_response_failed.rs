// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookResponseFailed` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookResponseFailedData,
};

/// Sent when a background response has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseFailed {
    /// The Unix timestamp (in seconds) of when the model response failed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookResponseFailedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `response.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
