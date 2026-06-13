// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseDone` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaResponse,
};

/// Returned when a Response is done streaming. Always emitted, no matter the final state. The Response
/// object included in the `response.done` event will include all output Items in the Response but will
/// omit the raw audio data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub response: RealtimeBetaResponse,
}
