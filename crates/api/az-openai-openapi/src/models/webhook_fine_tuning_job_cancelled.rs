// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WebhookFineTuningJobCancelled` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookFineTuningJobCancelledData,
};

/// Sent when a fine-tuning job has been cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFineTuningJobCancelled {
    /// The Unix timestamp (in seconds) of when the fine-tuning job was cancelled.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookFineTuningJobCancelledData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `fine_tuning.job.cancelled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
