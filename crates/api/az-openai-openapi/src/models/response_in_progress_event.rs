// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// Emitted when the response is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInProgressEvent {
    /// The type of the event. Always `response.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The response that is in progress.
    pub response: Response,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
