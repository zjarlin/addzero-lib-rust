// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookBatchCompleted` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookBatchCompletedData,
};

/// Sent when a batch API request has been completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchCompleted {
    /// The Unix timestamp (in seconds) of when the batch API request was completed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookBatchCompletedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `batch.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
