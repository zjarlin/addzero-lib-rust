// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookBatchFailed` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookBatchFailedData,
};

/// Sent when a batch API request has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchFailed {
    /// The Unix timestamp (in seconds) of when the batch API request failed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookBatchFailedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `batch.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
