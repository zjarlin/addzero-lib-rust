// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookFineTuningJobFailed` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookFineTuningJobFailedData,
};

/// Sent when a fine-tuning job has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFineTuningJobFailed {
    /// The Unix timestamp (in seconds) of when the fine-tuning job failed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookFineTuningJobFailedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `fine_tuning.job.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
