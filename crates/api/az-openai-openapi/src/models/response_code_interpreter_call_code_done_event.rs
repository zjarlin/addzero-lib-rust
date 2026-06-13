// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseCodeInterpreterCallCodeDoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the code snippet is finalized by the code interpreter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCodeInterpreterCallCodeDoneEvent {
    /// The type of the event. Always `response.code_interpreter_call_code.done`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response for which the code is finalized.
    pub output_index: i32,
    /// The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    /// The final code snippet output by the code interpreter.
    pub code: String,
    /// The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
}
