// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseCodeInterpreterCallCodeDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a partial code snippet is streamed by the code interpreter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCodeInterpreterCallCodeDeltaEvent {
    /// The type of the event. Always `response.code_interpreter_call_code.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response for which the code is being streamed.
    pub output_index: i32,
    /// The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    /// The partial code snippet being streamed by the code interpreter.
    pub delta: String,
    /// The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
}
