// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseQueuedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// Emitted when a response is queued and waiting to be processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseQueuedEvent {
    /// The type of the event. Always 'response.queued'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The full response object that is queued.
    pub response: Response,
    /// The sequence number for this event.
    pub sequence_number: i32,
}
