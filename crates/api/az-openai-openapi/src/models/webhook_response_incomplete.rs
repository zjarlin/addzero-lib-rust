// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookResponseIncomplete` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookResponseIncompleteData,
};

/// Sent when a background response has been interrupted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseIncomplete {
    /// The Unix timestamp (in seconds) of when the model response was interrupted.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookResponseIncompleteData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `response.incomplete`.
    #[serde(rename = "type")]
    pub type_value: String,
}
