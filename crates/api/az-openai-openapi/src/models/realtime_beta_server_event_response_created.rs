// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaResponse,
};

/// Returned when a new Response is created. The first event of response creation, where the response is
/// in an initial state of `in_progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseCreated {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.created`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub response: RealtimeBetaResponse,
}
