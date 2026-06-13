// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeBetaServerEventResponseFunctionCallArgumentsDelta` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the model-generated function call arguments are updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseFunctionCallArgumentsDelta {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.function_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the response.
    pub response_id: String,
    /// The ID of the function call item.
    pub item_id: String,
    /// The index of the output item in the response.
    pub output_index: i32,
    /// The ID of the function call.
    pub call_id: String,
    /// The arguments delta as a JSON string.
    pub delta: String,
}
