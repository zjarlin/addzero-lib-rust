// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookResponseCancelled` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookResponseCancelledData,
};

/// Sent when a background response has been cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseCancelled {
    /// The Unix timestamp (in seconds) of when the model response was cancelled.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookResponseCancelledData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `response.cancelled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
