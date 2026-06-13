// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFunctionCallArgumentsDeltaEvent` DTO.

use serde::{Deserialize, Serialize};

/// Emitted when there is a partial function-call arguments delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFunctionCallArgumentsDeltaEvent {
    /// The type of the event. Always `response.function_call_arguments.delta`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the output item that the function-call arguments delta is added to.
    pub item_id: String,
    /// The index of the output item that the function-call arguments delta is added to.
    pub output_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The function-call arguments delta that is added.
    pub delta: String,
}
