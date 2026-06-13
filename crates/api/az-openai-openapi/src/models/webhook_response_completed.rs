// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookResponseCompleted` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookResponseCompletedData,
};

/// Sent when a background response has been completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseCompleted {
    /// The Unix timestamp (in seconds) of when the model response was completed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookResponseCompletedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `response.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
