// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseCodeInterpreterCallCompletedEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the code interpreter call is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCodeInterpreterCallCompletedEvent {
    /// The type of the event. Always `response.code_interpreter_call.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response for which the code interpreter call is completed.
    pub output_index: i32,
    /// The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    /// The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
}
