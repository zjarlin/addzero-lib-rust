// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseCodeInterpreterCallInterpretingEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when the code interpreter is actively interpreting the code snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCodeInterpreterCallInterpretingEvent {
    /// The type of the event. Always `response.code_interpreter_call.interpreting`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response for which the code interpreter is interpreting code.
    pub output_index: i32,
    /// The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    /// The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
}
