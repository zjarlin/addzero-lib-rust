// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeClientEventSessionUpdate` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeClientEventSessionUpdateSession,
};

/// Send this event to update the session’s configuration. The client may send this event at any time to
/// update any field except for `voice` and `model`. `voice` can be updated only if there have been no
/// other audio outputs yet. When the server receives a `session.update`, it will respond with a
/// `session.updated` event showing the full, effective configuration. Only the fields that are present
/// in the `session.update` are updated. To clear a field like `instructions`, pass an empty string. To
/// clear a field like `tools`, pass an empty array. To clear a field like `turn_detection`, pass
/// `null`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeClientEventSessionUpdate {
    /// Optional client-generated ID used to identify this event. This is an arbitrary string that a client
    /// may assign. It will be passed back if there is an error with the event, but the corresponding
    /// `session.updated` event will not include it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The event type, must be `session.update`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Update the Realtime session. Choose either a realtime session or a transcription session.
    pub session: RealtimeClientEventSessionUpdateSession,
}
