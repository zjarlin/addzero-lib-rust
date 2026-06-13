// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookEvalRunCanceled` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookEvalRunCanceledData,
};

/// Sent when an eval run has been canceled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunCanceled {
    /// The Unix timestamp (in seconds) of when the eval run was canceled.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookEvalRunCanceledData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `eval.run.canceled`.
    #[serde(rename = "type")]
    pub type_value: String,
}
