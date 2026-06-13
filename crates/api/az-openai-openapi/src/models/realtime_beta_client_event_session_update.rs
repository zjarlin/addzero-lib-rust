// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaClientEventSessionUpdate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateRequest,
};

/// Send this event to update the session’s default configuration. The client may send this event at any
/// time to update any field, except for `voice`. However, note that once a session has been initialized
/// with a particular `model`, it can’t be changed to another model using `session.update`. When the
/// server receives a `session.update`, it will respond with a `session.updated` event showing the full,
/// effective configuration. Only the fields that are present are updated. To clear a field like
/// `instructions`, pass an empty string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaClientEventSessionUpdate {
    /// Optional client-generated ID used to identify this event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub session: RealtimeSessionCreateRequest,
}
