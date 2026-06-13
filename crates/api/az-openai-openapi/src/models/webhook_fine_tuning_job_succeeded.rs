// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookFineTuningJobSucceeded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookFineTuningJobSucceededData,
};

/// Sent when a fine-tuning job has succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFineTuningJobSucceeded {
    /// The Unix timestamp (in seconds) of when the fine-tuning job succeeded.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookFineTuningJobSucceededData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `fine_tuning.job.succeeded`.
    #[serde(rename = "type")]
    pub type_value: String,
}
