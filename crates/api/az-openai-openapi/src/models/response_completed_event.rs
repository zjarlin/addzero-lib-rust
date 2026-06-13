// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// Emitted when the model response is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCompletedEvent {
    /// The type of the event. Always `response.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Properties of the completed response.
    pub response: Response,
    /// The sequence number for this event.
    pub sequence_number: i32,
}
