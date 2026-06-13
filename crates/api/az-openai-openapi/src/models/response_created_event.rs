// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseCreatedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// An event that is emitted when a response is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCreatedEvent {
    /// The type of the event. Always `response.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The response that was created.
    pub response: Response,
    /// The sequence number for this event.
    pub sequence_number: i32,
}
