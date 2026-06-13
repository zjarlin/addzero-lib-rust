// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookEvalRunSucceeded` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    WebhookEvalRunSucceededData,
};

/// Sent when an eval run has succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvalRunSucceeded {
    /// The Unix timestamp (in seconds) of when the eval run succeeded.
    pub created_at: i64,
    /// The unique ID of the event.
    pub id: String,
    /// Event data payload.
    pub data: WebhookEvalRunSucceededData,
    /// The object of the event. Always `event`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// The type of the event. Always `eval.run.succeeded`.
    #[serde(rename = "type")]
    pub type_value: String,
}
