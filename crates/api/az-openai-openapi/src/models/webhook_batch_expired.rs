// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookBatchExpired` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookBatchExpiredData,
};

/// Sent when a batch API request has expired.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchExpired {
    /// The Unix timestamp (in seconds) of when the batch API request expired.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookBatchExpiredData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `batch.expired`.
    #[serde(rename = "type")]
    pub type_value: String,
}
