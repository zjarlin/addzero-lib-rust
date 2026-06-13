// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseIncompleteEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Response,
};

/// An event that is emitted when a response finishes as incomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseIncompleteEvent {
    /// The type of the event. Always `response.incomplete`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The response that was incomplete.
    pub response: Response,
    /// The sequence number of this event.
    pub sequence_number: i32,
}
