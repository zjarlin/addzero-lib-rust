// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFailedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// An event that is emitted when a response fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFailedEvent {
    /// The type of the event. Always `response.failed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The response that failed.
    pub response: Response,
}
