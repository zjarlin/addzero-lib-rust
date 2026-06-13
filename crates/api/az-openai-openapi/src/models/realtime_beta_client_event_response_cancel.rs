// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaClientEventResponseCancel` DTO.

use serde::{Deserialize, Serialize};

/// Send this event to cancel an in-progress response. The server will respond with a `response.done`
/// event with a status of `response.status=cancelled`. If there is no response to cancel, the server
/// will respond with an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventResponseCancel {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `response.cancel`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// A specific response ID to cancel - if not provided, will cancel an in-progress response in the
    /// default conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
}
