// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseCodeInterpreterCallInProgressEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when a code interpreter call is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCodeInterpreterCallInProgressEvent {
    /// The type of the event. Always `response.code_interpreter_call.in_progress`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The index of the output item in the response for which the code interpreter call is in progress.
    pub output_index: i32,
    /// The unique identifier of the code interpreter tool call item.
    pub item_id: String,
    /// The sequence number of this event, used to order streaming events.
    pub sequence_number: i32,
}
