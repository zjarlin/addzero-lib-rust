// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventResponseFunctionCallArgumentsDone` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the model-generated function call arguments are done streaming. Also emitted when a
/// Response is interrupted, incomplete, or cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventResponseFunctionCallArgumentsDone {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `response.function_call_arguments.done`.
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
    /// The name of the function that was called.
    pub name: String,
    /// The final arguments as a JSON string.
    pub arguments: String,
}
