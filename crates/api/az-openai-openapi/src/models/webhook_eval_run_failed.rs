// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookEvalRunFailed` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookEvalRunFailedData,
};

/// Sent when an eval run has failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunFailed {
    /// The Unix timestamp (in seconds) of when the eval run failed.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookEvalRunFailedData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `eval.run.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
}
